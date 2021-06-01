fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("resources/hekk.exe.manifest");
    res.compile()
        .expect("failed to compile windows resource file");
}
