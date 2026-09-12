from fastapi import FastAPI, Request

app = FastAPI()


@app.get("/")
async def get_root(request: Request):
    return dict(request.query_params)


@app.put("/")
async def put_root(request: Request):
    return dict(request.query_params)


@app.post("/send_msg_form")
async def send_msg_form(request: Request):
    form = await request.form()
    return dict(form)


@app.post("/send_msg_json")
async def send_msg_json(request: Request):
    return await request.json()


@app.post("/send_msg_with_query")
async def send_msg_with_query(request: Request):
    form = await request.form()

    return {
        **dict(request.query_params),
        **dict(form),
    }