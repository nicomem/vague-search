use std::{io::Write, mem::size_of, path::Path, ptr::null_mut, slice};

#[derive(Debug, Copy, Clone)]
struct Header {
    vec_len: u64,
}

#[derive(Debug)]
struct Node {
    a: u16,
    b: bool,
    c: [u64; 3],
}

#[derive(Debug)]
struct Data<'a> {
    ptr: *const u8,
    ptr_len: usize,
    pub nodes: &'a [Node],
}

const HEADER_LEN: usize = std::mem::size_of::<Header>();

fn timer<T>(name: &str, f: impl FnOnce() -> T) -> T {
    let instant = std::time::Instant::now();
    let r = f();
    let dt = instant.elapsed();
    println!("Timer {} : {:?}", name, dt);
    r
}

fn main() {
    let arg_path = std::env::args().nth(1).expect("No dict given");
    let path = Path::new(&arg_path);
    dbg!(&path);

    {
        let data_write = (1..)
            .map(|i| Node {
                a: i,
                b: i.is_power_of_two(),
                c: [i as u64, 2 * i as u64, 3 * i as u64],
            })
            .take(3_000_000)
            .collect::<Vec<_>>();

        println!("Writing...");
        timer("mmap_write", || mmap_write(&path, &data_write));
    }

    {
        println!("Reading...");
        let data_read = timer("mmap_read", || mmap_read(&path));
        println!("Read: {} entries", data_read.nodes.len());
        timer("Get every i/10 * len() of data", || {
            for i in (0..10).map(|i| i * data_read.nodes.len() / 10) {
                let _data = data_read.nodes.get(i);
                // dbg!(_data);
            }
        });
    }
}

trait AsBytes {
    unsafe fn as_u8_slice(&'_ self) -> &'_ [u8];
}

impl<T: Sized> AsBytes for [T] {
    unsafe fn as_u8_slice(&'_ self) -> &'_ [u8] {
        slice::from_raw_parts(
            self.as_ptr() as *const T as *const u8,
            self.len() * size_of::<T>(),
        )
    }
}

impl AsBytes for Header {
    unsafe fn as_u8_slice(&'_ self) -> &'_ [u8] {
        slice::from_raw_parts((self as *const Header) as *const u8, size_of::<Header>())
    }
}

fn mmap_write(f: &Path, data: &[Node]) {
    // Write header
    let header = Header {
        vec_len: data.len() as u64,
    };

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(f)
        .unwrap();
    let bytes = unsafe { header.as_u8_slice() };
    file.write_all(bytes).unwrap();

    let bytes = unsafe { data.as_u8_slice() };

    // Write data
    // dbg!(&data.bytes);
    file.write_all(&bytes).unwrap();

    // let mut s = String::new();
    // std::fs::File::open(f)
    //     .unwrap()
    //     .read_to_string(&mut s)
    //     .unwrap();
    // dbg!(s);
}

fn mmap_read(path: &Path) -> Data {
    let ptr;
    let nodes;
    let file_length = path.metadata().unwrap().len();
    unsafe {
        let fd = libc::open(path.to_str().unwrap().as_ptr() as *const i8, libc::O_RDONLY);
        // dbg!(fd);
        ptr = libc::mmap(
            null_mut(),
            file_length as usize,
            libc::PROT_READ,
            libc::MAP_SHARED,
            fd,
            0,
        ) as *const u8;
        libc::close(fd);

        // dbg!(ptr);

        let header = *(ptr as *const Header);
        let data_ptr = ptr.offset(HEADER_LEN as isize);

        nodes = slice::from_raw_parts(data_ptr as *const Node, header.vec_len as usize);
    }

    Data {
        ptr,
        ptr_len: file_length as usize,
        nodes,
    }

    // let mut file = std::fs::File::open(f).unwrap();
    // let mut buf_header = [0u8; HEADER_LEN];
    // timer("read_header", || file.read_exact(&mut buf_header).unwrap());

    // let header: Header = unsafe { std::mem::transmute(buf_header) };
    // // dbg!(header);

    // let mut buf_data = vec![0u8; header.vec_len as usize];
    // timer("read_data", || file.read_exact(&mut buf_data).unwrap());

    // // dbg!(&buf_data);

    // let data = Data { bytes: buf_data };
    // data

    // unsafe {
    //     let fd = libc::open(f.to_str().unwrap().as_ptr() as *const i8, libc::O_RDONLY);

    //     // Read header
    //     let ptr = libc::mmap(
    //         libc::PT_NULL as *mut libc::c_void,
    //         HEADER_LEN,
    //         libc::PROT_READ,
    //         libc::MAP_PRIVATE,
    //         fd,
    //         0,
    //     );

    //     header = *(ptr as *mut Header);

    //     libc::munmap(ptr, HEADER_LEN);

    //     // Read data
    //     let len_bytes = header.vec_len as usize * std::mem::size_of::<u8>();
    //     let ptr = libc::mmap(
    //         libc::PT_NULL as *mut libc::c_void,
    //         len_bytes,
    //         libc::PROT_READ,
    //         libc::MAP_PRIVATE,
    //         fd,
    //         HEADER_LEN as i64,
    //     );

    //     data = Data {
    //         bytes: Vec::from_raw_parts(
    //             ptr as *mut u8,
    //             header.vec_len as usize,
    //             header.vec_len as usize,
    //         ),
    //     };

    //     libc::close(fd);
    // };
}
