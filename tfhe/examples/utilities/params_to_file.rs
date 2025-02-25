use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use tfhe::boolean::parameters::{BooleanParameters, VEC_BOOLEAN_PARAM};
use tfhe::core_crypto::commons::parameters::{GlweDimension, LweDimension, PolynomialSize};
use tfhe::core_crypto::prelude::{DynamicDistribution, TUniform, UnsignedInteger};
use tfhe::keycache::NamedParam;
use tfhe::shortint::parameters::classic::compact_pk::ALL_PARAMETER_VEC_COMPACT_PK;
use tfhe::shortint::parameters::classic::gaussian::ALL_PARAMETER_VEC_GAUSSIAN;
use tfhe::shortint::parameters::compact_public_key_only::p_fail_2_minus_64::ks_pbs::V0_11_PARAM_PKE_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M64;
use tfhe::shortint::parameters::multi_bit::ALL_MULTI_BIT_PARAMETER_VEC;
use tfhe::shortint::parameters::{
    CompactPublicKeyEncryptionParameters, CompressionParameters, ShortintParameterSet,
    COMP_PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M64,
    PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M64,
};

pub trait ParamDetails<T: UnsignedInteger> {
    fn lwe_dimension(&self) -> LweDimension;
    fn glwe_dimension(&self) -> GlweDimension;
    fn lwe_noise_distribution(&self) -> DynamicDistribution<T>;
    fn glwe_noise_distribution(&self) -> DynamicDistribution<T>;
    fn polynomial_size(&self) -> PolynomialSize;
    fn log_ciphertext_modulus(&self) -> usize;
}

impl ParamDetails<u32> for BooleanParameters {
    fn lwe_dimension(&self) -> LweDimension {
        self.lwe_dimension
    }

    fn glwe_dimension(&self) -> GlweDimension {
        self.glwe_dimension
    }

    fn lwe_noise_distribution(&self) -> DynamicDistribution<u32> {
        self.lwe_noise_distribution
    }
    fn glwe_noise_distribution(&self) -> DynamicDistribution<u32> {
        self.glwe_noise_distribution
    }

    fn polynomial_size(&self) -> PolynomialSize {
        self.polynomial_size
    }

    fn log_ciphertext_modulus(&self) -> usize {
        32
    }
}

impl ParamDetails<u64> for ShortintParameterSet {
    fn lwe_dimension(&self) -> LweDimension {
        self.lwe_dimension()
    }

    fn glwe_dimension(&self) -> GlweDimension {
        self.glwe_dimension()
    }

    fn lwe_noise_distribution(&self) -> DynamicDistribution<u64> {
        self.lwe_noise_distribution()
    }
    fn glwe_noise_distribution(&self) -> DynamicDistribution<u64> {
        self.glwe_noise_distribution()
    }

    fn polynomial_size(&self) -> PolynomialSize {
        self.polynomial_size()
    }

    fn log_ciphertext_modulus(&self) -> usize {
        assert!(self.ciphertext_modulus().is_native_modulus());
        64
    }
}

impl ParamDetails<u64> for CompactPublicKeyEncryptionParameters {
    fn lwe_dimension(&self) -> LweDimension {
        self.encryption_lwe_dimension
    }

    fn glwe_dimension(&self) -> GlweDimension {
        panic!("glwe_dimension not applicable for compact public-key encryption parameters")
    }

    fn lwe_noise_distribution(&self) -> DynamicDistribution<u64> {
        self.encryption_noise_distribution
    }
    fn glwe_noise_distribution(&self) -> DynamicDistribution<u64> {
        panic!(
            "glwe_noise_distribution not applicable for compact public-key encryption parameters"
        )
    }

    fn polynomial_size(&self) -> PolynomialSize {
        panic!("polynomial_size not applicable for compact public-key encryption parameters")
    }

    fn log_ciphertext_modulus(&self) -> usize {
        assert!(self.ciphertext_modulus.is_native_modulus());
        64
    }
}

impl ParamDetails<u64> for CompressionParameters {
    fn lwe_dimension(&self) -> LweDimension {
        panic!("lwe_dimension not applicable for compression parameters")
    }

    fn glwe_dimension(&self) -> GlweDimension {
        self.packing_ks_glwe_dimension
    }

    fn lwe_noise_distribution(&self) -> DynamicDistribution<u64> {
        panic!("lwe_noise_distribution not applicable for compression parameters")
    }
    fn glwe_noise_distribution(&self) -> DynamicDistribution<u64> {
        self.packing_ks_key_noise_distribution
    }

    fn polynomial_size(&self) -> PolynomialSize {
        self.packing_ks_polynomial_size
    }

    fn log_ciphertext_modulus(&self) -> usize {
        64
    }
}

#[derive(Eq, PartialEq)]
enum ParametersFormat {
    Lwe,
    Glwe,
    LweGlwe,
}

///Function to print in the lattice_estimator format the parameters
/// Format:   LWE.Parameters(n=722, q=2^32, Xs=ND.UniformMod(2),
/// Xe=ND.DiscreteGaussian(56139.60810663548), tag='test_lattice_estimator')
pub fn format_lwe_parameters_to_lattice_estimator<
    U: UnsignedInteger,
    T: ParamDetails<U> + NamedParam,
>(
    param: &T,
) -> String {
    let name = param.name();

    match param.lwe_noise_distribution() {
        DynamicDistribution::Gaussian(distrib) => {
            let modular_std_dev =
                param.log_ciphertext_modulus() as f64 + distrib.standard_dev().0.log2();

            format!(
                "{}_LWE = LWE.Parameters(\n n = {},\n q ={},\n Xs=ND.UniformMod(2), \n Xe=ND.DiscreteGaussian({}),\n tag='{}_lwe' \n)\n\n",
                name, param.lwe_dimension().0, (1u128<<param.log_ciphertext_modulus() as u128), 2.0_f64.powf(modular_std_dev), name)
        }
        DynamicDistribution::TUniform(distrib) => {
            format!(
                "{}_LWE = LWE.Parameters(\n n = {},\n q ={},\n Xs=ND.Uniform(0,1), \n Xe=ND.DiscreteGaussian({}),\n tag='{}_lwe' \n)\n\n",
                name, param.lwe_dimension().0, (1u128<<param.log_ciphertext_modulus() as u128), tuniform_equivalent_gaussian_std_dev(&distrib), name)
        }
    }
}

///Function to print in the lattice_estimator format the parameters
/// Format: LWE.Parameters(n=722, q=2^32, Xs=ND.UniformMod(2),
/// Xe=ND.DiscreteGaussian(56139.60810663548), tag='test_lattice_estimator')
pub fn format_glwe_parameters_to_lattice_estimator<
    U: UnsignedInteger,
    T: ParamDetails<U> + NamedParam,
>(
    param: &T,
) -> String {
    let name = param.name();

    match param.glwe_noise_distribution() {
        DynamicDistribution::Gaussian(distrib) => {
            let modular_std_dev =
                param.log_ciphertext_modulus() as f64 + distrib.standard_dev().0.log2();

            format!(
                "{}_GLWE = LWE.Parameters(\n n = {},\n q = {},\n Xs=ND.UniformMod(2), \n Xe=ND.DiscreteGaussian({}),\n tag='{}_glwe' \n)\n\n",
                name, param.glwe_dimension().0 * param.polynomial_size().0, (1u128<<param.log_ciphertext_modulus() as u128), 2.0_f64.powf(modular_std_dev), name)
        }
        DynamicDistribution::TUniform(distrib) => {
            format!(
                "{}_GLWE = LWE.Parameters(\n n = {},\n q ={},\n Xs=ND.Uniform(0,1), \n Xe=ND.DiscreteGaussian({}),\n tag='{}_glwe' \n)\n\n",
                name, param.glwe_dimension().0 * param.polynomial_size().0, (1u128<<param.log_ciphertext_modulus() as u128), tuniform_equivalent_gaussian_std_dev(&distrib), name)
        }
    }
}

fn tuniform_equivalent_gaussian_std_dev<U: UnsignedInteger>(distribution: &TUniform<U>) -> f64 {
    f64::sqrt((2_f64.powf(2.0 * distribution.bound_log2() as f64 + 1_f64) + 1_f64) / 6_f64)
}

fn write_file(file: &mut File, filename: &Path, line: impl Into<String>) {
    let error_message = format!("unable to write file {}", filename.to_str().unwrap());
    file.write_all(line.into().as_bytes())
        .expect(&error_message);
}

fn write_all_params_in_file<U: UnsignedInteger, T: ParamDetails<U> + Copy + NamedParam>(
    filename: &str,
    params: &[T],
    format: ParametersFormat,
) {
    let path = Path::new(filename);
    File::create(path).expect("create results file failed");
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .expect("cannot open parsed results file");

    for params in params.iter() {
        if format == ParametersFormat::LweGlwe || format == ParametersFormat::Lwe {
            write_file(
                &mut file,
                path,
                format_lwe_parameters_to_lattice_estimator(params),
            );
        }

        if format == ParametersFormat::LweGlwe || format == ParametersFormat::Glwe {
            write_file(
                &mut file,
                path,
                format_glwe_parameters_to_lattice_estimator(params),
            );
        }
    }
    write_file(&mut file, path, "all_params = [\n");
    for params in params.iter() {
        if format == ParametersFormat::LweGlwe || format == ParametersFormat::Lwe {
            let param_lwe_name = format!("{}_LWE,", params.name());
            write_file(&mut file, path, param_lwe_name);
        }

        if format == ParametersFormat::LweGlwe || format == ParametersFormat::Glwe {
            let param_glwe_name = format!("{}_GLWE,", params.name());
            write_file(&mut file, path, param_glwe_name);
        }
    }
    write_file(&mut file, path, "\n]\n");
}

fn main() {
    let work_dir = std::env::current_dir().unwrap();
    let mut new_work_dir = work_dir;
    new_work_dir.push("ci");
    std::env::set_current_dir(new_work_dir).unwrap();

    write_all_params_in_file(
        "boolean_parameters_lattice_estimator.sage",
        &VEC_BOOLEAN_PARAM,
        ParametersFormat::LweGlwe,
    );

    let all_classic_pbs = [
        ALL_PARAMETER_VEC_GAUSSIAN.to_vec(),
        ALL_PARAMETER_VEC_COMPACT_PK.to_vec(),
        vec![PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M64],
    ]
    .concat();
    let classic_pbs = all_classic_pbs
        .iter()
        .map(|p| ShortintParameterSet::from(*p))
        .collect::<Vec<_>>();
    write_all_params_in_file(
        "shortint_classic_parameters_lattice_estimator.sage",
        &classic_pbs,
        ParametersFormat::LweGlwe,
    );

    let multi_bit_pbs = ALL_MULTI_BIT_PARAMETER_VEC
        .iter()
        .map(|p| ShortintParameterSet::from(*p))
        .collect::<Vec<_>>();
    write_all_params_in_file(
        "shortint_multi_bit_parameters_lattice_estimator.sage",
        &multi_bit_pbs,
        ParametersFormat::LweGlwe,
    );

    write_all_params_in_file(
        "shortint_cpke_parameters_lattice_estimator.sage",
        &[V0_11_PARAM_PKE_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M64],
        ParametersFormat::Lwe,
    );

    write_all_params_in_file(
        "shortint_list_compression_parameters_lattice_estimator.sage",
        &[COMP_PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M64],
        ParametersFormat::Glwe,
    );

    // TODO perform this gathering later
    // let wopbs = ALL_PARAMETER_VEC_WOPBS
    //     .iter()
    //     .map(|p| ShortintParameterSet::from(*p))
    //     .collect::<Vec<_>>();
    // write_all_params_in_file(
    //     "shortint_wopbs_parameters_lattice_estimator.sage",
    //     &wopbs,
    // );
}
