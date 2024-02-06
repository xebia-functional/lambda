# Setup

The build process relies on [`npm`](https://www.npmjs.com/),
[`esbuild`](https://esbuild.github.io/), and the TypeScript compiler. You only
need to install `npm` manually; it will take care of installing the rest of the
toolchain.

# Building

## `events-a`

Change the current working directory to `events-a`.

You can build the eponymous service like so:

```shell
npm install
npm run build
npm run postbuild
```

You can use the custom `create` script to create and publish the remote AWS
Lambda, but you will likely need to modify the `--role` parameter. Once
customized for your AWS environment, you can:

```shell
npm run create
```

If you need to redeploy (because you made changes):

```shell
npm run package
npm run deploy
```

## `events-b`

Change the current working directory to `events-b`.

You can build the eponymous service like so:

```shell
npm install
npm run build
npm run postbuild
```

You can use the custom `create` script to create and publish the remote AWS
Lambda, but you will likely need to modify the `--role` parameter. Once
customized for your AWS environment, you can:

```shell
npm run create
```

If you need to redeploy (because you made changes):

```shell
npm run package
npm run deploy
```
