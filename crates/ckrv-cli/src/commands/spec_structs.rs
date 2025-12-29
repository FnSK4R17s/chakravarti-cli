
#[derive(Debug, serde::Serialize, serde::Deserialize, tabled::Tabled)]
struct Task {
    #[tabled(rename = "ID")]
    id: String,
    
    #[tabled(rename = "Phase")]
    phase: String,

    #[tabled(rename = "Title")]
    title: String,
    
    #[tabled(skip)]
    description: String,
    
    #[tabled(skip)]
    file: Option<String>,
    
    #[tabled(skip)]
    user_story: Option<String>,
    
    #[tabled(skip)]
    parallel: bool,
    
    #[tabled(rename = "Status")]
    status: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TaskFile {
    tasks: Vec<Task>,
}
