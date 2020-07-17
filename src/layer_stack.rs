/// A [LIFO data structure](https://en.wikipedia.org/wiki/Stack_(abstract_data_type))
/// working on layers instead of elements.
///
/// A layer is a sized contiguous collection of data, also synonymous to a [slice](std::slice).
/// Each layer is also represented by a character.
/// The string resulted of the concatenation of all layers character can then be queried.
pub struct LayerStack<E, S: Copy + Into<usize>> {
    elements: Vec<E>,
    layers: Vec<S>,
    word: String,
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
            word: String::with_capacity(cap_layers),
        }
    }

    /// Clear the stack of all elements. Do not deallocate its storage.
    pub fn clear(&mut self) {
        self.elements.clear();
        self.layers.clear();
    }

    /// Retrieve the string resulted of the concatenation of all layers character.
    pub fn get_layers_word(&self) -> &str {
        &self.word
    }

    /// Create a new layer of the wanted size in the stack and return it.
    ///
    /// The layer_char can be ommited for the first layer, it will not be added
    /// to the concatenated string.
    /// It is however **mandotary** for any other layer.
    ///
    /// The last pushed is also accessible by calling the
    /// [fetch_layer](Self::fetch_layer) method.
    pub fn push_layer(&mut self, layer_char: Option<char>, size: S) -> &mut [E]
    where
        E: Default,
    {
        let layer_start = self.elements.len();

        // Add the layer's character to the inner string
        if let Some(ch) = layer_char {
            self.word.push(ch);
        } else {
            // No layer_char can be given only for the first layer
            debug_assert_eq!(self.layers.len(), 0);
        }

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

            // Remove the layer's character from the inner string
            self.word.pop();
            true
        } else {
            false
        }
    }

    /// Return the last pushed layer as a mutable slice.
    pub fn fetch_layer(&self) -> Option<&[E]> {
        if let Some(&size) = self.layers.last() {
            let nb_elements = self.elements.len();
            Some(&self.elements[nb_elements - size.into()..])
        } else {
            None
        }
    }

    /// Try to fetch the last 3 layers as mutable slices.
    ///
    /// Depending on the number of layers in the stack, the tuple can contain
    /// empty slices:
    /// - \>= 3 elements: `[&mut cur_layer, &mut last_layer, &mut parent_layer]`
    /// - \>= 2 elements: `[&mut cur_layer, &mut last_layer, []]`
    /// - 1 element: `[&mut cur_layer, [], []]`
    /// - 0 element: `[[], [], []]`
    ///
    /// Those empty slices are still returned as mutable but since they have
    /// a size of 0, they cannot be modified.
    pub fn fetch_last_3_layers(&mut self) -> [&mut [E]; 3] {
        match self.layers[..] {
            [.., psize, lsize, csize] => {
                // If at least 2 elements, get the last 2 layers sizes
                // and find their start indices
                let nb_elements = self.elements.len();
                let i_cur_layer = nb_elements - csize.into();
                let i_last_layer = i_cur_layer - lsize.into();
                let i_parent_layer = i_last_layer - psize.into();

                // Split elements into two mutable slices at the index of the last slice
                let (all_previous, cur_layer) = self.elements.split_at_mut(i_cur_layer);
                let (all_previous, last_layer) = all_previous.split_at_mut(i_last_layer);
                let parent_layer = &mut all_previous[i_parent_layer..];

                // Return the slices of the two last layers
                [cur_layer, last_layer, parent_layer]
            }

            [lsize, csize] => {
                // If at least 2 elements, get the last 2 layers sizes
                // and find their start indices
                let nb_elements = self.elements.len();
                let i_cur_layer = nb_elements - csize.into();
                let i_last_layer = i_cur_layer - lsize.into();

                // Split elements into two mutable slices at the index of the last slice
                let (all_previous, cur_layer) = self.elements.split_at_mut(i_cur_layer);
                let last_layer = &mut all_previous[i_last_layer..];

                // Return the slices of the two last layers
                [cur_layer, last_layer, Default::default()]
            }

            // If only one element, return the entire elements slice
            // which correspond to the last and only layer
            [_] => [
                &mut self.elements[..],
                Default::default(),
                Default::default(),
            ],

            // If the stack is empty, return an empty slice
            [] => Default::default(),
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
        assert_eq!(stack.get_layers_word(), "");

        let layer = stack.push_layer(None, u8::MAX);
        assert_eq!(layer.len(), u8::MAX as usize);
        assert_eq!(stack.get_layers_word(), "");
        assert_eq!(stack.pop_layer(), true);

        let layer = stack.push_layer(Some('a'), 0);
        assert_eq!(layer.len(), 0);
        assert_eq!(stack.get_layers_word(), "a");
        assert_eq!(stack.pop_layer(), true);

        stack.push_layer(Some('b'), 14);
        let layer = stack.fetch_layer();
        assert!(layer.is_some());
        assert_eq!(layer.unwrap().len(), 14);
        assert_eq!(stack.get_layers_word(), "b");
        assert_eq!(stack.pop_layer(), true);

        assert_eq!(stack.get_layers_word(), "");
        assert_eq!(stack.pop_layer(), false);
        assert!(stack.fetch_layer().is_none());
    }

    #[test]
    pub fn test_many_layers() {
        let mut stack = LayerStack::with_capacity(1000, 100);
        assert_eq!(stack.pop_layer(), false);
        assert!(stack.fetch_layer().is_none());

        for len in 0..=1000 {
            let layer = stack.push_layer(Some('a'), len);
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

            assert_eq!(stack.get_layers_word().chars().count(), len + 1);
            assert_eq!(stack.pop_layer(), true);
        }

        assert!(stack.fetch_layer().is_none());
        assert_eq!(stack.pop_layer(), false);
    }

    #[test]
    pub fn test_fetch_last_3_layers() {
        let mut stack = LayerStack::<u8, u8>::with_capacity(0, 0);

        // Test empty stack
        let [cur, last, parent] = stack.fetch_last_3_layers();
        assert_eq!(cur.len(), 0);
        assert_eq!(last.len(), 0);
        assert_eq!(parent.len(), 0);
        assert_eq!(stack.get_layers_word(), "");

        // Test one element
        stack.push_layer(Some('c'), 5);
        let [cur, last, parent] = stack.fetch_last_3_layers();
        assert_eq!(cur.len(), 5);
        assert_eq!(last.len(), 0);
        assert_eq!(parent.len(), 0);
        assert_eq!(stack.get_layers_word(), "c");

        // Test two elements
        stack.push_layer(Some('a'), 10);
        let [cur, last, parent] = stack.fetch_last_3_layers();
        assert_eq!(cur.len(), 10);
        assert_eq!(last.len(), 5);
        assert_eq!(parent.len(), 0);
        assert_eq!(stack.get_layers_word(), "ca");

        // Test three elements
        stack.push_layer(Some('r'), 1);
        let [cur, last, parent] = stack.fetch_last_3_layers();
        assert_eq!(cur.len(), 1);
        assert_eq!(last.len(), 10);
        assert_eq!(parent.len(), 5);
        assert_eq!(stack.get_layers_word(), "car");

        // Pop last (2 elements)
        stack.pop_layer();
        let [cur, last, parent] = stack.fetch_last_3_layers();
        assert_eq!(cur.len(), 10);
        assert_eq!(last.len(), 5);
        assert_eq!(parent.len(), 0);
        assert_eq!(stack.get_layers_word(), "ca");

        // Pop last (1 element)
        stack.pop_layer();
        let [cur, last, parent] = stack.fetch_last_3_layers();
        assert_eq!(cur.len(), 5);
        assert_eq!(last.len(), 0);
        assert_eq!(parent.len(), 0);
        assert_eq!(stack.get_layers_word(), "c");

        // Pop last (empty)
        stack.pop_layer();
        let [cur, last, parent] = stack.fetch_last_3_layers();
        assert_eq!(cur.len(), 0);
        assert_eq!(last.len(), 0);
        assert_eq!(parent.len(), 0);
        assert_eq!(stack.get_layers_word(), "");
    }
}
