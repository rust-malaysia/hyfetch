use pyo3::Python;

fn main() {
    Python::with_gil(|py| {
        py.run_bound(r#"
        try:
            import qwqfetch
            # distro_detector only return a bash variable
            # so we use qwqfetch builtin distro detector
            print(qwqfetch.get_ascres(asc))  
        except ImportError as e:  # module not found etc
            print("qwqfetch is not installed. Install it by executing:")  # use print to output hint directly
            print("pip install git+https://github.com/nexplorer-3e/qwqfetch")  # TODO: public repo
            exit(127)
        "#, None, None);
    });
}
