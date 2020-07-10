/// A [LIFO data structure](https://en.wikipedia.org/wiki/Stack_(abstract_data_type))
/// working on layers instead of elements.
///
/// A layer is a sized contiguous collection of data, also synonymous to a [slice](std::slice).
pub struct LayerStack<E, S: Copy + Into<usize>> {
    elements: Vec<E>,
    layers: Vec<S>,
}

impl<E, S: Copy + Into<usize>> LayerStack<E, S> {
    /// Constructs a new, empty LayerStack<T> with the specified capacities.
    ///
    /// It will be able to hold the following capacities without reallocating its inner storage:
    /// - `cap_elements` elements
    /// - `cap_layers` layers
    pub fn with_capacity(cap_elements: usize, cap_layers: usize) -> Self {
        Self {
            elements: Vec::with_capacity(cap_elements),
            layers: Vec::with_capacity(cap_layers),
        }
    }

    /// Clear the stack of all elements. Do not deallocate its storage.
    pub fn clear(&mut self) {
        self.elements.clear();
        self.layers.clear();
    }

    /// Create a new layer of the wanted size in the stack and return it.
    ///
    /// The last pushed is also accessible by calling the
    /// [fetch_layer](Self::fetch_layer) method.
    pub fn push_layer(&mut self, size: S) -> &mut [E]
    where
        E: Default,
    {
        let layer_start = self.elements.len();

        // Create the requested number of elements
        self.elements
            .resize_with(layer_start + size.into(), E::default);

        // Save the size of the new layer
        self.layers.push(size);

        // Return the new layer
        &mut self.elements[layer_start..]
    }

    /// Delete the last layer if the stack if the stack is not empty.
    ///
    /// Return whether a layer was popped.
    pub fn pop_layer(&mut self) -> bool {
        // Get and remove the last layer size
        if let Some(size) = self.layers.pop() {
            // Remove the last `size` elements corresponding to the popped layer
            self.elements.truncate(self.elements.len() - size.into());
            true
        } else {
            false
        }
    }

    /// Return the last pushed layer as a mutable slice.
    pub fn fetch_layer(&mut self) -> Option<&mut [E]> {
        if let Some(&size) = self.layers.last() {
            let nb_elements = self.elements.len();
            Some(&mut self.elements[nb_elements - size.into()..])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_one_layer() {
        let mut stack = LayerStack::<u16, u8>::with_capacity(0, 0);
        assert_eq!(stack.pop_layer(), false);
        assert!(stack.fetch_layer().is_none());

        let layer = stack.push_layer(u8::MAX);
        assert_eq!(layer.len(), u8::MAX as usize);
        assert_eq!(stack.pop_layer(), true);

        let layer = stack.push_layer(0);
        assert_eq!(layer.len(), 0);
        assert_eq!(stack.pop_layer(), true);

        stack.push_layer(14);
        let layer = stack.fetch_layer();
        assert!(layer.is_some());
        assert_eq!(layer.unwrap().len(), 14);
        assert_eq!(stack.pop_layer(), true);

        assert_eq!(stack.pop_layer(), false);
        assert!(stack.fetch_layer().is_none());
    }

    #[test]
    pub fn test_many_layers() {
        let mut stack = LayerStack::with_capacity(1000, 100);
        assert_eq!(stack.pop_layer(), false);
        assert!(stack.fetch_layer().is_none());

        for len in 0..=1000 {
            let layer = stack.push_layer(len);
            for i in 0..len {
                layer[i as usize] = i;
            }
        }

        for len in (0..=1000).rev() {
            let layer_opt = stack.fetch_layer();
            assert!(layer_opt.is_some());

            let layer = layer_opt.unwrap();
            assert_eq!(layer.len(), len);

            for i in 0..len {
                assert_eq!(layer[i], i);
            }
            assert_eq!(stack.pop_layer(), true);
        }

        assert!(stack.fetch_layer().is_none());
        assert_eq!(stack.pop_layer(), false);
    }
}
