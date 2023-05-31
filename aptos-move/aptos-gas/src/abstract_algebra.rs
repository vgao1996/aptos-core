use crate::{InternalGas, InternalGasPerArg};
use move_core_types::gas_algebra::{Arg, GasQuantity, InternalGasUnit, NumArgs, UnitDiv};
use std::{
    fmt,
    marker::PhantomData,
    ops::{Add, Mul},
};

struct GasParameters {
    sum: SumGasParameters,
}

struct SumGasParameters {
    base: InternalGas,
    per_element: InternalGasPerArg,
}

enum SUM_BASE {}

enum SUM_PER_ELEMENT {}

trait AbstractGasFormula {
    type Unit;

    fn materialize(&self, gas_params: &GasParameters) -> GasQuantity<Self::Unit>;

    fn visit(&self, visitor: &mut impl AbstractGasFormulaVisitor);
}

trait AbstractGasFormulaVisitor {
    fn add(&mut self);

    fn mul(&mut self);

    fn gas_param<P>(&mut self);

    fn quantity<U>(&mut self, quantity: GasQuantity<U>);
}

#[derive(Debug)]
struct AbstractGasAdd<L, R> {
    left: L,
    right: R,
}

#[derive(Debug)]
struct AbstractGasMul<L, R> {
    left: L,
    right: R,
}

struct GetGasParam<P> {
    phantom: PhantomData<P>,
}

impl AbstractGasFormula for GetGasParam<SUM_BASE> {
    type Unit = InternalGasUnit;

    #[inline]
    fn materialize(&self, gas_params: &GasParameters) -> GasQuantity<Self::Unit> {
        gas_params.sum.base
    }

    #[inline]
    fn visit(&self, visitor: &mut impl AbstractGasFormulaVisitor) {
        visitor.gas_param::<SUM_BASE>();
    }
}

impl AbstractGasFormula for GetGasParam<SUM_PER_ELEMENT> {
    type Unit = UnitDiv<InternalGasUnit, Arg>;

    #[inline]
    fn materialize(&self, gas_params: &GasParameters) -> GasQuantity<Self::Unit> {
        gas_params.sum.per_element
    }

    #[inline]
    fn visit(&self, visitor: &mut impl AbstractGasFormulaVisitor) {
        visitor.gas_param::<SUM_PER_ELEMENT>();
    }
}

impl<U> AbstractGasFormula for GasQuantity<U> {
    type Unit = U;

    #[inline]
    fn materialize(&self, gas_params: &GasParameters) -> GasQuantity<Self::Unit> {
        *self
    }

    #[inline]
    fn visit(&self, visitor: &mut impl AbstractGasFormulaVisitor) {
        visitor.quantity(*self)
    }
}

impl<P, R> Add<R> for GetGasParam<P>
where
    Self: AbstractGasFormula,
    R: AbstractGasFormula,
{
    type Output = AbstractGasAdd<Self, R>;

    #[inline]
    fn add(self, rhs: R) -> Self::Output {
        AbstractGasAdd {
            left: self,
            right: rhs,
        }
    }
}

impl<P, R> Mul<R> for GetGasParam<P>
where
    Self: AbstractGasFormula,
    R: AbstractGasFormula,
{
    type Output = AbstractGasMul<Self, R>;

    #[inline]
    fn mul(self, rhs: R) -> Self::Output {
        AbstractGasMul {
            left: self,
            right: rhs,
        }
    }
}

impl<L, R, U> AbstractGasFormula for AbstractGasAdd<L, R>
where
    L: AbstractGasFormula<Unit = U>,
    R: AbstractGasFormula<Unit = U>,
{
    type Unit = U;

    #[inline]
    fn materialize(&self, gas_params: &GasParameters) -> GasQuantity<Self::Unit> {
        self.left.materialize(gas_params) + self.right.materialize(gas_params)
    }

    #[inline]
    fn visit(&self, visitor: &mut impl AbstractGasFormulaVisitor) {
        self.left.visit(visitor);
        self.right.visit(visitor);
        visitor.add();
    }
}

impl<L, R, UL, UR, O> AbstractGasFormula for AbstractGasMul<L, R>
where
    L: AbstractGasFormula<Unit = UL>,
    R: AbstractGasFormula<Unit = UR>,
    GasQuantity<UL>: Mul<GasQuantity<UR>, Output = GasQuantity<O>>,
{
    type Unit = O;

    #[inline]
    fn materialize(&self, gas_params: &GasParameters) -> GasQuantity<Self::Unit> {
        self.left.materialize(gas_params) * self.right.materialize(gas_params)
    }

    #[inline]
    fn visit(&self, visitor: &mut impl AbstractGasFormulaVisitor) {
        self.left.visit(visitor);
        self.right.visit(visitor);
        visitor.mul();
    }
}

trait GasCore {
    fn charge(&mut self, abstract_amount: impl AbstractGasFormula) -> anyhow::Result<()>;
}

macro_rules! gas_param {
    ($param_ty: ty) => {
        GetGasParam::<$param_ty> {
            phantom: PhantomData,
        }
    };
}

fn native_sum(gas_core: &mut impl GasCore, v: &[u64]) -> anyhow::Result<u64> {
    gas_core.charge(
        gas_param!(SUM_BASE) + gas_param!(SUM_PER_ELEMENT) * NumArgs::new(v.len() as u64),
    )?;

    Ok(v.iter().sum())
}

struct ConcreteGasCore {
    gas_params: GasParameters,
}

struct AbstractGasCore;

impl GasCore for ConcreteGasCore {
    fn charge(&mut self, abstract_amount: impl AbstractGasFormula) -> anyhow::Result<()> {
        let amount = abstract_amount.materialize(&self.gas_params);
        println!("charge {}", amount);
        Ok(())
    }
}

struct PrintVisitor;

impl AbstractGasFormulaVisitor for PrintVisitor {
    fn add(&mut self) {
        print!(" +");
    }

    fn mul(&mut self) {
        print!(" *");
    }

    fn gas_param<P>(&mut self) {
        let tn = std::any::type_name::<P>().split("::");
        print!(" {}", tn.last().unwrap());
    }

    fn quantity<U>(&mut self, quantity: GasQuantity<U>) {
        print!(" {}", quantity);
    }
}

impl GasCore for AbstractGasCore {
    fn charge(&mut self, abstract_amount: impl AbstractGasFormula) -> anyhow::Result<()> {
        abstract_amount.visit(&mut PrintVisitor);
        println!();
        Ok(())
    }
}

#[test]
fn demo_abstract_gas_algebra() {
    let mut gas_core = ConcreteGasCore {
        gas_params: GasParameters {
            sum: SumGasParameters {
                base: 10.into(),
                per_element: 1.into(),
            },
        },
    };
    native_sum(&mut gas_core, &[1, 2, 3]).unwrap();

    let mut gas_core = AbstractGasCore;
    native_sum(&mut gas_core, &[1, 2, 3]).unwrap();
}
