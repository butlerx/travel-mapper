from os import path

from fastapi.templating import Jinja2Templates

dir_path = path.dirname(path.realpath(__file__))


templates = Jinja2Templates(directory=dir_path)
