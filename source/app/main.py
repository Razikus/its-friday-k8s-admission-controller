from fastapi import FastAPI, Request
import calendar
from datetime import datetime
import logging
import os


app = FastAPI()
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("itsfriday-controller")
PODNAME = os.environ.get("APP_POD_NAME", "NOT_A_POD")

MYFRIDAYS = [ int(x) for x in os.environ.get("MY_FRIDAYS", "4").split(",") ]

@app.get("/health")
async def health():
    return {"message": "OK"}

@app.get("/ready")
async def ready():
    return {"message": "OK"}

def getResponse(uid: str, status: bool, apiVersion: str, message: str):
    code = 403
    if status == True:
        code = 200
    response = {
        "apiVersion": apiVersion,
        "kind": "AdmissionReview",
        "response": {
            "uid": uid,
            "allowed": status,
            "status": {
                "code": code,
                "message": message
            }
        }
    }
    return response


@app.post("/validate")
async def validate(req: Request):
    ISFRIDAY = datetime.today().weekday() in MYFRIDAYS
    DAYOFWEEK = calendar.day_name[datetime.today().weekday()]
    jsoned = await req.json()
    reqOfUid = jsoned["request"]["uid"]
    logger.info(f"Received request {reqOfUid}")
    ownerReferences = jsoned["request"].get("object", dict()).get("metadata", dict()).get("ownerReferences", [])
    apiVersion = jsoned.get("apiVersion", "admission.k8s.io/v1")
    if(len(ownerReferences) > 0):
        logger.info(f"{PODNAME}: Accepted request {reqOfUid}, owner references found (automated job)")
        return getResponse(reqOfUid, True, apiVersion, "Owner found, automaticaly scheduled")
    if(ISFRIDAY):
        logger.info(f"{PODNAME}: Rejected request {reqOfUid}, ITS {DAYOFWEEK}!")
        return getResponse(reqOfUid, False, apiVersion, "ITS {DAYOFWEEK}!")
    else:
        logger.info(f"{PODNAME}: Accepted request {reqOfUid}, Its not friday (nor any other blocked day)!")
        return getResponse(reqOfUid, True, apiVersion, "Its not a blocked day. Go!")

