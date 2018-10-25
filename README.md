the binary in rsrv is a musl compiled via docker with
golddranks/rust_musl_docker:nightly-2018-10-20

the build script just looks for a postgres image and echos it into .env

todo: clean up code, streamline nightly rust musl builds, come up with stuff to
      actually use this with :)
