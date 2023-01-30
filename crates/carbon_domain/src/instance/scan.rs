use crate::try_path_fmt;
use log::trace;
use std::io;
use std::path::{Path, PathBuf};
use tokio::fs::{read_dir, DirEntry};

use crate::instance::configuration::{parse_from_file, ConfigurationFileParsingError};
use crate::instance::consts::{CONFIGURATION_FILE_RELATIVE_PATH, MINECRAFT_PACKAGE_RELATIVE_PATH};
use crate::instance::scan::InstanceScanError::{
    FileStructureDoesNotMatch, FolderStructureDoesNotMatch, PathNotIsNotPointingToAFolder,
};
use crate::instance::{Instance, InstanceStatus};
use futures::{future, StreamExt};
use thiserror::Error;
use tokio_stream::wrappers::ReadDirStream;

use super::Instances;

type InstanceScanResult = Result<Vec<Result<Instance, InstanceScanError>>, InstanceScanError>;
type InstanceTestResult = Result<(), InstanceScanError>;

#[derive(Error, Debug)]
pub enum InstanceScanError {
    #[error("path `{path}` does not contain any valid instance at ")]
    NoInstancesInFolder {
        path: PathBuf,
        recursive_searched: bool,
    },

    #[error("path `{0}` does not point to a directory ")]
    PathNotIsNotPointingToAFolder(PathBuf),

    #[error("io error: {0} !\n")]
    IoError(#[from] io::Error),

    #[error("expected folder `{0}` but not found! \n")]
    FolderStructureDoesNotMatch(PathBuf),

    #[error("expected file `{0}` but not found! \n")]
    FileStructureDoesNotMatch(PathBuf),

    #[error("error happened while parsing instances configuration file: `{0}`\n")]
    ConfigurationFileParsingError(#[from] ConfigurationFileParsingError),
}

impl Instances {
    pub async fn scan_for_instances(&mut self) -> InstanceScanResult {
        let instances_path = self.instances_path.clone().join("instances");
        // todo : add recursive mode
        trace!(
            "scanning directory {} for instances",
            try_path_fmt!(instances_path)
        );
        let res = match instances_path.is_dir() {
            true => Ok(future::join_all(
                ReadDirStream::new(read_dir(instances_path).await?)
                    .map(scan_for_instances_single_directory)
                    .collect::<Vec<_>>()
                    .await,
            )
            .await),
            false => {
                trace!(
                    "path {} is not pointing to a directory! aborting instance scan process ...",
                    try_path_fmt!(instances_path)
                );
                Err(PathNotIsNotPointingToAFolder(instances_path.to_path_buf()))
            }
        };

        if let Ok(instances) = res {
            for instance in instances.into_iter() {
                if let Ok(instance) = instance {
                    self.instances.insert(instance);
                }
            }
        }

        unimplemented!()
    }
}

async fn scan_for_instances_single_directory(
    directory_path: Result<DirEntry, io::Error>,
) -> Result<Instance, InstanceScanError> {
    let directory_path = &directory_path?.path();
    trace!(
        "scanning directory {} for instance",
        try_path_fmt!(directory_path.as_path())
    );
    check_instance_directory_sanity(directory_path).await?;
    let configuration_file_path = Path::new(directory_path).join(CONFIGURATION_FILE_RELATIVE_PATH);
    let instance_configuration_file = parse_from_file(&configuration_file_path).await?;
    Ok(Instance {
        name: instance_configuration_file.instance_name.clone(),
        // Can safely unwrap as we probably don't support non-utf8 paths
        uuid: directory_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        minecraft_package: instance_configuration_file
            .minecraft_package_configuration
            .into(),
        persistence_status: InstanceStatus::Persisted(directory_path.to_path_buf()),
    })
}

pub async fn check_instance_directory_sanity<T: AsRef<Path> + Sync>(
    target_instance_directory_path: &T,
) -> InstanceTestResult {
    let target_instance_directory_path = target_instance_directory_path.as_ref();
    trace!(
        "start check process for folder {}",
        try_path_fmt!(target_instance_directory_path)
    );
    let minecraft_package_path =
        PathBuf::from(target_instance_directory_path).join(MINECRAFT_PACKAGE_RELATIVE_PATH);
    let instance_configuration_file_path =
        PathBuf::from(target_instance_directory_path).join(CONFIGURATION_FILE_RELATIVE_PATH);
    let instance_configuration_file_exist =
        instance_configuration_file_path.exists() && instance_configuration_file_path.is_file();
    let minecraft_package_folder_exist =
        minecraft_package_path.exists() && minecraft_package_path.is_dir();
    match (
        instance_configuration_file_exist,
        minecraft_package_folder_exist,
    ) {
        (false, _) => Err(FolderStructureDoesNotMatch(
            instance_configuration_file_path,
        )),
        (_, false) => Err(FileStructureDoesNotMatch(minecraft_package_path)),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod unit_tests {
    #[test]
    fn test_instance_scan_ok() {}

    #[test]
    fn test_instance_scan_err() {}

    #[test]
    fn test_directory_structure_check_ok() {
        /*let res = InstanceScanner::check_directory_structure(&PathBuf::from("test_snapshots").join("instance_example")).await;
        let affirmative_check = matches!(res, Ok(_));
        assert!(affirmative_check);*/
    }

    #[test]
    fn test_directory_structure_check_err() {
        /*let res = InstanceScanner::check_directory_structure(&PathBuf::from("test_snapshots").join("malformed_instance_example")).await;
        let denial_check = matches!(res, Err(_)); // todo : add every error case
        assert!(denial_check);*/
    }
}