//! Compile les ressources GResource (icône de l'application).

fn main() {
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/mondashboard.gresource.xml",
        "mondashboard.gresource",
    );
}
