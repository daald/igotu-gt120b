import dagger
import random
from typing import Annotated
from dagger import DefaultPath, Doc, dag, function, object_type


@object_type
class IgotuGt120:
    ## https://docs.dagger.io/quickstart/ci/#initialize-a-dagger-module
    @function
    def container_echo(self, string_arg: str) -> dagger.Container:
        """Returns a container that echoes whatever string argument is provided"""
        return dag.container().from_("alpine:latest").with_exec(["echo", string_arg])

    @function
    async def grep_dir(self, directory_arg: dagger.Directory, pattern: str) -> str:
        """Returns lines that match a pattern in the files of the provided Directory"""
        return await (
            dag.container()
            .from_("alpine:latest")
            .with_mounted_directory("/mnt", directory_arg)
            .with_workdir("/mnt")
            .with_exec(["grep", "-R", pattern, "."])
            .stdout()
        )


    @function
    async def build(
        self,
        source: Annotated[
            dagger.Directory, DefaultPath("/"), Doc("hello-dagger source directory")
        ],
    ) -> dagger.Container:
        """Build the application container"""
        return await (
            self.build_env(source)
            .with_mounted_directory("/src", source)
            #.with_exec(["cargo", "install", "--path", "."])
            .with_exec(["cargo", "build"])
            .directory("target")
            # use: dagger -c 'build | export target'
            # see: https://docs.dagger.io/cookbook#builds
        )

    @function
    def build_env(
        self,
        source: Annotated[
            dagger.Directory, DefaultPath("/"), Doc("hello-dagger source directory")
        ],
    ) -> dagger.Container:
        """Build a ready-to-use development environment"""
        #cargo_cache = dag.cache_volume("cargo")
        return (
            dag.container()
            .from_("rust")
            .with_workdir("/src")
            .with_file("Cargo.toml", source.file("Cargo.toml"))
            .with_exec(["sh", "-exc", """
            mkdir -p src/bin/
            echo 'fn main() {}' >src/bin/__empty.rs
            echo >>Cargo.toml
            echo '[[bin]]' >>Cargo.toml
            echo 'name = "__empty"' >>Cargo.toml
            cargo fetch
            rm -rv Cargo.toml src/bin/__empty.rs src/
            """])
            .with_directory("/src", source)
            #.with_mounted_cache("/root/.npm", cargo_cache)
            .with_workdir("/src")
            #.with_exec(["npm", "install"])
        )
