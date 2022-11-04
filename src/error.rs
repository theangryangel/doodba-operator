use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to serialize YAML: {source}")]
    YamlSerializationError {
        #[from]
        source: serde_yaml::Error,
    },

    #[error("Kubernetes reported error: {source}")]
    KubeError {
        #[from]
        source: kube::Error,
    },

    #[error("Finalizer Error: {0}")]
    FinalizerError(#[source] kube::runtime::finalizer::Error<kube::Error>),

    #[error("Kubernetes failed to delete object: {source}")]
    KubeDeleteError {
        #[from]
        source: kube::runtime::wait::delete::Error,
    },
}
