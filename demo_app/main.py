# Intentionally unsecured for testing purposes do not expose!

import os
from fastapi import FastAPI, Request, UploadFile, File

app = FastAPI()

UPLOAD_DIR = "uploads"
os.makedirs(UPLOAD_DIR, exist_ok=True)


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

# INSECURE FILE UPLOAD
@app.post("/upload_image")
async def upload_image(file: UploadFile = File(...)):
    file_location = os.path.join(UPLOAD_DIR, file.filename)
    with open(file_location, "wb") as f:
        f.write(await file.read())
    return {"filename": file.filename, "saved": file_location}
