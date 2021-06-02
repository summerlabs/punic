use std::fs;

pub fn scan_xcframeworks(output: String) -> Vec<String> {
    println!("Scanning frameworks in Carthage build folder...");
    let mut frameworks = vec![];
    for entry in fs::read_dir(output).unwrap() {
        let en = entry.unwrap();
        let path = en.path();
        if path.is_dir() && path.to_str().unwrap().contains("xcframework") {
            let path_string = path
                .to_str()
                .unwrap()
                .to_string()
                .split("/")
                .last()
                .unwrap()
                .to_string();
            println!("{}", path_string);
            frameworks.push(path_string);
        }
    }
    return frameworks;
}
