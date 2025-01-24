import os
from os.path import join
from pathlib import Path
from urllib.parse import quote_plus, unquote_plus

from robyn.templating import JinjaTemplate

templates = JinjaTemplate(join(Path(__file__).parent.resolve()))
templates.env.filters["quote_plus"] = lambda x: quote_plus(str(x)) if x else ""
templates.env.filters["unquote_plus"] = lambda x: unquote_plus(str(x)) if x else ""
