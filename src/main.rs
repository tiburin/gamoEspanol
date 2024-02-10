use gamo::apps::Sphere;
static WRITE_BUILD: bool = false;
static RUN_MAS: bool = true;
static WRITE_BOOKTORE: bool = false;

fn main() {
    // put line below in the empty input
    // aparter
    let name = "aparter";
    let sphere = Sphere::new().setup(name);

    let gamo = gamo::start("");
    if RUN_MAS {
        sphere.mas(name);
    }
    if WRITE_BOOKTORE {
        gamo::write_booktore(&gamo)
    }
    if WRITE_BUILD {
        gamo::write_build(&gamo, false);
    }
}
