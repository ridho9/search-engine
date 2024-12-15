from pathlib import Path
import os
import json
from typing import TypedDict
import trafilatura as trf
from tqdm import tqdm

from bs4 import BeautifulSoup
from multiprocessing import Process, Pool


class RawPage(TypedDict):
    url: str
    raw_html: str


class ProcessedPage(TypedDict):
    url: str
    title: str
    body: str
    desc: str


def process_output_file(fname):
    with (
        open(f"./output/{fname}", "r") as infile,
        open(f"./output-1/{fname}", "w") as outfile,
    ):
        print(f"process {fname}")

        for line in infile:
            if line.strip() == "":
                continue

            parsed: RawPage = json.loads(line)

            ext_text = trf.extract(parsed["raw_html"])
            ext_meta = trf.extract_metadata(parsed["raw_html"])

            html = BeautifulSoup(parsed["raw_html"], features="lxml")
            title = html.find("title")
            if title is None:
                title = ext_meta.title
            else:
                title = title.string  # type: ignore
            if title is not None:
                title = title.strip()

            text = html.get_text(separator="\n")
            bs4_text = "\n".join([x for x in text.splitlines() if x.strip() != ""])

            processed = ProcessedPage(
                url=parsed["url"],
                title=title,  # type: ignore
                body=ext_text or bs4_text,
            )

            json.dump(processed, outfile)
            outfile.write("\n")


if __name__ == "__main__":
    try:
        os.rmdir("./output-1")
    except:
        pass
    Path("./output-1").mkdir(exist_ok=True)

    file_list = list(Path("./output").glob("*"))

    with Pool() as pool:
        results = [
            pool.apply_async(process_output_file, (fname.name,)) for fname in file_list
        ]

        output = [r.get() for r in results]
