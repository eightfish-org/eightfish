# Introduction to EightFish Framework SDK

EightFish includes an elegant MVC development SDK.

This is a document for EightFish framework that introduce the features of the SDK of the framework. We will use the [example/simple](https://github.com/eightfish-org/eightfish/blob/master/examples/simple) as an example in this document.

## The App Entry

An EightFish app is actually a [Spin redis-triggered application](https://developer.fermyon.com/spin/redis-trigger). The entry of this type of application looks like:

``` #[redis_component]                                                              fn on_message(message: Bytes) -> Result<()> { 
```

You can look into the [example file](https://github.com/eightfish-org/eightfish/blob/master/examples/simple/src/lib.rs#L31) to learn the style.

The `on_message` function is just a boilerplate to comply with. In its body, we need to build the instance of our app, and mount it to `spin_worker::Worker`, and call `worker.work(message)` to process this incoming message.

Every time the message comes (from the redis channel), the EightFish App handler will process this message, and give response to somewhere (some caches, which is defined by components `spin_worker` and `http_gate` together, not the channel message comes from) in redis.

## The App Instance

Every EightFish App should create an EightFish App instance, like that:

```
pub fn build_app() -> EightFishApp {
    let mut sapp = EightFishApp::new();
    sapp.add_global_filter(Box::new(MyGlobalFilter))
        .add_module(Box::new(article::ArticleModule));

    sapp
}
```

The above function name is arbitrary, but should return the type of `EightFishApp`. In its body we create the instance of `EightFishApp`, mount global filter of the app and all modules implementing the application logic to this instance. 

## The Global Filter

The global filter of EightFish is for all incoming requests. Some actions (like cookie verifying, authentication, etc.) should be checked before any specific logic being executed. So we need this mechanism to tackle them.

```
impl GlobalFilter for MyGlobalFilter {
    fn before(&self, _req: &mut Request) -> EightFishResult<()> {
        Ok(())
    }

    fn after(&self, _req: &Request, _res: &mut Response) -> EightFishResult<()> {
        Ok(())
    }
}
```

`GlobalFilter` is a trait defined by EightFish SDK. And there are two methods in it: `before()` and `after()`. The `before()` is used to process request ahead of any other biz logic code, and the `after()` is used to process the response after all biz logic but before responding to user, it's the last step we can write something to attach or modify information to the response.

## The Module

Every specific biz logic should be put into each specific module, and mount these modules into the app instance described above. You can look into the [article.rs file](https://github.com/eightfish-org/eightfish/blob/master/examples/simple/src/article.rs) to get concrete picture of the structure of a module.

First, you need to define a struct type, like: 

```
pub struct ArticleModule;
```

and implement the `Module` trait on it:

```
impl Module for ArticleModule {
    fn router(&self, router: &mut Router) -> Result<()> {
        router.get("/article/:id", Self::get_one);
        router.post("/article/new", Self::new_article);
        router.post("/article/update", Self::update);
        router.post("/article/delete/:id", Self::delete);

        Ok(())
    }
}
```

The above snippet fills the url router `Router` by implementing the `fn router`, the router supports two kinds of methods: `get` and `post`, correspond to the HTTP GET and POST methods respectively.

Besides of that, in the trait `Module`, we can implement the second level filters: `before()` and `after()`. These two filters apply on the url matches within this local module, not affecting other urls in other modules. So it is a kind of filter in module level. The `EightFish::Module` is [defined](https://github.com/eightfish-org/eightfish/blob/master/src/app.rs#L41) as follow:

```
pub trait EightFishModule: Sync + Send {
    /// module before filter, will be executed before handler
    fn before(&self, _req: &mut EightFishRequest) -> Result<()> {
        Ok(())
    }

    /// module after filter, will be executed after handler
    fn after(&self, _req: &EightFishRequest, _res: &mut EightFishResponse) -> Result<()> {
        Ok(())
    }

    /// module router method, used to write router collection of this module here
    fn router(&self, router: &mut EightFishRouter) -> Result<()>;
}
```

## The URL Handler

The corresponding handlers are implemented onto the module struct.

```
fn handler_name(req: &mut Request) -> Result<Response> {
```

Every url handler has unified function signature: a `req: &mut Request` as parameter, and a `Result<Response>` as function returning.

In the handler function, you can process the request at the third level. 

## The Middleware Mechanism

In EightFish, middleware is just a normal function. It accepts the `req: &mut Request` and return `Result<()>`.

```
pub fn middleware_fn(req: &mut Request) -> Result<()> {
```

The middleware function could be placed in the global filter, module filter, or every handler function. This conduct a flexible three levels of middleware system.

## Initial Globals

There is a method on EightFishApp: [`init_global()`](https://github.com/eightfish-org/eightfish/blob/master/src/app.rs#L85). You can put the global variables which should exist as long as the EightFish app's lifecycle in it. This is a mechanism for shared data between different requests.

In the provided closure, you need to insert your desired data into the extension part of the request, as follows:

```

let a_global = Arc::new(Mutex::new(..))

...
app.init_global(|req: &mut Request| -> Result<()> {
	req.ext_mut().insert("a_global", a_global);
})
...
```


## The Helper Macros

EightFish designs some helper macros to improve the experience of logic coding, especially on interacting with SQL db.

EightFish provides a derived macro named `EightFishModel` for user's model.

For example, you just need to put this macro into the derive section above a model (struct) definition.

```
#[derive(Debug, Clone, Serialize, Deserialize, EightFishModel, Default)]
pub struct Article {
    id: String,
    title: String,
    content: String,
    authorname: String,
}
```

After that, the struct `Article` will gain some powerful functionalities like:

```
fn model_name() -> String {}
fn field_names() -> String {}
fn build_insert_sql() -> String {}
fn from_row(row: Vec<DbValue>) -> #ident {}
...
```

These functionalities make you write biz code easily, rapidly and happily.

You can refer to the detailed inline doc [here](https://github.com/eightfish-org/eightfish/blob/master/eightfish-derive/src/eight_fish_model.rs#L85).


## Run the App

After all, switch into the app directory, and type:

```
spin up
```

to run this app up.

