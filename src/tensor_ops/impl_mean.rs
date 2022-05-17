use crate::prelude::*;

pub trait HasMeanMethod: Tensor {
    fn mean(self) -> Tensor0D<Self::TapeHolder>;
}

impl<T: Tensor> HasMeanMethod for T
where
    Cpu: Device<T::Array>,
{
    fn mean(self) -> Tensor0D<Self::TapeHolder> {
        let result = Tensor0D::<NoTape>::new(Cpu::mean(self.data()));
        let deriv = Cpu::map(self.data(), |_| 1.0 / T::Array::NUM_ELEMENTS as f32);
        let (t, mut tape_holder) = self.split_tape_holder();
        let _result = result.phantom();
        tape_holder.add_operation(move |tape| {
            let g: &f32 = tape.ref_gradient(&_result);
            let d_grad = Cpu::map(&deriv, |v| v * g);
            Cpu::add_assign(tape.mut_gradient(&t), &d_grad);
        });
        result.with_tape_holder(tape_holder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean_0d() {
        let t: Tensor0D = Tensor0D::new(3.0);
        let r = t.trace().mean();
        assert_eq!(r.data(), &3.0);
        let gradients = backward(r);
        assert_eq!(gradients.ref_gradient(&t), &1.0);
    }

    #[test]
    fn test_mean_1d() {
        let t: Tensor1D<3> = Tensor1D::new([1.0, 2.0, 3.0]);
        let r: Tensor0D<WithTape> = t.trace().mean();
        assert_eq!(r.data(), &2.0);
        let gradients = backward(r);
        assert_eq!(gradients.ref_gradient(&t), &[1.0 / 3.0; 3]);
    }

    #[test]
    fn test_mean_2d() {
        let t: Tensor2D<2, 3> = Tensor2D::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        let r: Tensor0D<WithTape> = t.trace().mean();
        assert_eq!(r.data(), &3.5);
        let gradients = backward(r);
        assert_eq!(gradients.ref_gradient(&t), &[[1.0 / 6.0; 3]; 2]);
    }

    #[test]
    fn test_mean_3d() {
        let t: Tensor3D<4, 2, 3> = Tensor3D::ones();
        let r: Tensor0D<WithTape> = t.trace().mean();
        assert_eq!(r.data(), &1.0);
        let gradients = backward(r);
        assert_eq!(gradients.ref_gradient(&t), &[[[1.0 / 24.0; 3]; 2]; 4]);
    }
}
