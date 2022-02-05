use super::super::{utils::value_type_from_wasmparser, FuncTypeIdx, ModuleResources};
use crate::{core::ValueType, ModuleError};
use core::slice;

/// The type of a Wasm control flow block.
#[derive(Debug, Copy, Clone)]
pub struct BlockType {
    inner: BlockTypeInner,
}

/// The inner workings of the [`BlockType`].
#[derive(Debug, Copy, Clone)]
pub enum BlockTypeInner {
    /// A block type with no parameters and no results.
    Empty,
    /// A block type with no parameters and exactly one result.
    Returns(ValueType),
    /// A general block type with parameters and results.
    FuncType(FuncTypeIdx),
}

impl TryFrom<wasmparser::TypeOrFuncType> for BlockType {
    type Error = ModuleError;

    fn try_from(type_or_func_type: wasmparser::TypeOrFuncType) -> Result<Self, Self::Error> {
        let block_type = match type_or_func_type {
            wasmparser::TypeOrFuncType::Type(wasmparser::Type::EmptyBlockType) => Self::empty(),
            wasmparser::TypeOrFuncType::Type(return_type) => {
                let return_type = value_type_from_wasmparser(&return_type)?;
                Self::returns(return_type)
            }
            wasmparser::TypeOrFuncType::FuncType(func_type) => {
                Self::func_type(FuncTypeIdx(func_type))
            }
        };
        Ok(block_type)
    }
}

impl BlockType {
    /// Creates a [`BlockType`] from the underlying type.
    fn from_inner(inner: BlockTypeInner) -> Self {
        Self { inner }
    }

    /// Creates a [`BlockType`] with no parameter and no results.
    fn empty() -> Self {
        Self::from_inner(BlockTypeInner::Empty)
    }

    /// Creates a [`BlockType`] with no parameters and a single result type.
    fn returns(return_type: ValueType) -> Self {
        Self::from_inner(BlockTypeInner::Returns(return_type))
    }

    /// Creates a [`BlockType`] with parameters and results.
    pub(crate) fn func_type(func_type: FuncTypeIdx) -> Self {
        Self::from_inner(BlockTypeInner::FuncType(func_type))
    }

    /// Returns the parameter types of the [`BlockType`].
    pub fn params<'a>(&self, res: ModuleResources<'a>) -> &'a [ValueType] {
        match &self.inner {
            BlockTypeInner::Empty | BlockTypeInner::Returns(_) => &[],
            BlockTypeInner::FuncType(func_type) => res.get_func_type(*func_type).params(),
        }
    }

    /// Returns the result types of the [`BlockType`].
    pub fn results<'a, 'b, 'c>(&'a self, res: ModuleResources<'b>) -> &'c [ValueType]
    where
        'a: 'c,
        'b: 'c,
    {
        match &self.inner {
            BlockTypeInner::Empty => &[],
            BlockTypeInner::Returns(return_type) => slice::from_ref(return_type),
            BlockTypeInner::FuncType(func_type) => res.get_func_type(*func_type).results(),
        }
    }

    /// Returns the parameter and result types of the [`BlockType`].
    ///
    /// # Note
    ///
    /// This can sometimes be superior than using [`BlockType::params`] and
    /// [`BlockType::results`] separately due to lifetime constraints. Also it
    /// should be insignificantly more efficient.
    pub fn params_results<'a, 'b, 'c>(
        &'a self,
        res: ModuleResources<'b>,
    ) -> (&'b [ValueType], &'c [ValueType])
    where
        'a: 'c,
        'b: 'c,
    {
        match &self.inner {
            BlockTypeInner::Empty => (&[], &[]),
            BlockTypeInner::Returns(return_type) => (&[], slice::from_ref(return_type)),
            BlockTypeInner::FuncType(func_type) => res.get_func_type(*func_type).params_results(),
        }
    }
}