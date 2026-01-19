
//! Builder patterns for creating packed data structures

use std::marker::PhantomData;

/// Builder for creating packed data arrays with a fluent API
pub struct PackedDataBuilder<T> {
    data: Vec<T>,
    _phantom: PhantomData<T>,
}

impl<T> PackedDataBuilder<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            _phantom: PhantomData,
        }
    }

    pub fn push(mut self, item: T) -> Self {
        self.data.push(item);
        self
    }

    pub fn extend<I: IntoIterator<Item = T>>(mut self, items: I) -> Self {
        self.data.extend(items);
        self
    }

    pub fn build(self) -> Vec<T> {
        self.data
    }
}

impl<T> Default for PackedDataBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for building entities with fixed-point coordinates
#[derive(Debug)]
pub struct EntityBuilder<T> {
    items: Vec<T>,
}

impl<T> EntityBuilder<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(mut self, item: T) -> Self {
        self.items.push(item);
        self
    }

    pub fn extend(mut self, items: Vec<T>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn try_add<E>(mut self, item: Result<T, E>) -> Result<Self, E> {
        match item {
            Ok(v) => {
                self.items.push(v);
                Ok(self)
            }
            Err(e) => Err(e),
        }
    }

    pub fn build(self) -> Vec<T> {
        self.items
    }

    pub fn build_result<E>(self, _phantom: Option<E>) -> Result<Vec<T>, E> {
        // Simple wrapper: always Ok here; we could add validation if needed
        Ok(self.items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Clone)]
    struct Dummy(u32, u32);

    #[test]
    fn test_entity_builder_try_add() -> Result<(), Box<dyn std::error::Error>> {
        // Start with an empty builder
        let err = EntityBuilder::<Dummy>::new()
            .try_add::<&'static str>(Ok(Dummy(10, 20)))?
            .try_add::<&'static str>(Ok(Dummy(30, 40)))?
            .try_add::<&'static str>(Err("fail"))
            .unwrap_err();
        assert_eq!(err, "fail");

        // Collect items into a Vec
        let result: Vec<Dummy> = EntityBuilder::<Dummy>::new()
            .try_add::<&'static str>(Ok(Dummy(1, 2)))?
            .try_add::<&'static str>(Ok(Dummy(3, 4)))?
            .build();
        assert_eq!(result, vec![Dummy(1, 2), Dummy(3, 4)]);

        Ok(())
    }

    #[test]
    fn test_entity_builder_extend() {
        let items = vec![Dummy(5, 6), Dummy(7, 8)];
        let builder = EntityBuilder::<Dummy>::new().extend(items.clone());
        assert_eq!(builder.items, items);
    }
}
