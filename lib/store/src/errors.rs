error_chain!{
    errors {
        ProjectAlreadyArchived(project: String) {
            description("project is already archived")
                display("project '{}' is already archived", project)
        }
        NoProjectWithThisName(project: String) {
            description("no project with this name")
                display("no project with the name '{}' found", project)
        }
        NoteHasEmptyValue {
            description("not has empty value")
                display("note has empty value")
        }
    }
}
