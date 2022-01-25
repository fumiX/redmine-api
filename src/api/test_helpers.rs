use std::error::Error;
use tracing::trace;

use crate::api::projects::{CreateProject, DeleteProject, GetProject, Project, ProjectWrapper};

pub fn with_project<F>(name: &str, f: F) -> Result<(), Box<dyn Error>>
where
    F: FnOnce(&crate::api::Redmine, &str) -> Result<(), Box<dyn Error>>,
{
    dotenv::dotenv()?;
    let redmine = crate::api::Redmine::from_env()?;
    let get_endpoint = GetProject::builder().project_id_or_name(name).build()?;
    let get_result = redmine.json_response_body::<_, ProjectWrapper<Project>>(&get_endpoint);
    trace!("Get result in {} test:\n{:?}", name, get_result);
    if get_result.is_ok() {
        let delete_endpoint = DeleteProject::builder().project_id_or_name(name).build()?;
        redmine.ignore_response_body::<_>(&delete_endpoint)?;
    }
    let create_endpoint = CreateProject::builder()
        .name(format!("Unittest redmine-api {}", name))
        .identifier(name)
        .build()?;
    redmine.json_response_body::<_, ProjectWrapper<Project>>(&create_endpoint)?;
    let _fb = finally_block::finally(|| {
        trace!(%name, "Deleting test project");
        let delete_endpoint = DeleteProject::builder()
            .project_id_or_name(name)
            .build()
            .expect(&format!("Building delete enedpoint for {} failed", name));
        redmine
            .ignore_response_body::<_>(&delete_endpoint)
            .expect(&format!("Delete project {} failed", name));
    });
    trace!(%name, "Actual test body starts here");
    f(&redmine, name)?;
    trace!(%name, "Actual test body ends here");
    Ok(())
}