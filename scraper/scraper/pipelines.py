# Define your item pipelines here
#
# Don't forget to add your pipeline to the ITEM_PIPELINES setting
# See: https://docs.scrapy.org/en/latest/topics/item-pipeline.html


# useful for handling different item types with a single interface
from itemadapter.adapter import ItemAdapter
import trafilatura as trf
from bs4 import BeautifulSoup
import pymongo
import os
from scrapy.exceptions import DropItem
import requests
from readabilipy import simple_json_from_html_string


class DedupPipeline:
    def __init__(self):
        self.found_url = set()

    def process_item(self, item, spider):
        adapter = ItemAdapter(item)
        url = adapter.get("url")
        if url in self.found_url:
            raise DropItem(f"drop duplicate {url}")
        self.found_url.add(url)
        return item


class ExtractPipeline:
    def process_item(self, item, spider):
        adapter = ItemAdapter(item)

        raw_html: str = adapter.get("raw_html")  # type: ignore

        # ext_text = trf.extract(raw_html)
        # ext_meta = trf.extract_metadata(raw_html)

        # html = BeautifulSoup(raw_html, features="lxml")
        # title = html.find("title")
        # if title is None:
        #     title = ext_meta.title
        # else:
        #     title = title.string  # type: ignore
        # if title is not None:
        #     title = title.strip()

        # text = html.get_text(separator="\n")
        # bs4_text = "\n".join([x for x in text.splitlines() if x.strip() != ""])
        cleaned_data = simple_json_from_html_string(raw_html, use_readability=True)
        plain_text = [t["text"] for t in cleaned_data["plain_text"]]  # type: ignore
        # print(plain_text)

        result = dict(
            url=adapter.get("url"),
            title=cleaned_data["title"],  # type: ignore
            body=plain_text,
        )
        return result
        # return item


class MongoPipeline:
    collection_name = "page_items"

    def __init__(self, mongo_uri, mongo_db):
        self.mongo_uri = mongo_uri
        self.mongo_db = mongo_db

    @classmethod
    def from_crawler(cls, crawler):
        return cls(
            mongo_uri=os.getenv("MONGO_URI"),
            mongo_db=os.getenv("MONGO_DB"),
        )

    def open_spider(self, spider):
        self.client = pymongo.MongoClient(self.mongo_uri)
        self.db = self.client[self.mongo_db]
        self.collection = self.db[self.collection_name]
        self.collection.create_index("url", unique=True)

    def close_spider(self, spider):
        self.client.close()

    def process_item(self, item, spider):
        to_insert = ItemAdapter(item).asdict()
        result = self.collection.insert_one(to_insert)
        item["_id"] = result.inserted_id
        return item


class IndexPipeline:
    BATCH_SIZE = 100

    # def process_item(self, item, spider):
    #     adapter = ItemAdapter(item)
    #     host = os.getenv("ENGINE_HOST") or "http://localhost:3000"
    #     resp = requests.post(host + "/api/docs", json=adapter.asdict())
    #     print(f"insert {adapter.get("url")} {resp.status_code} {resp.text}")
    def open_spider(self, spider):
        self.items = []

    def process_item(self, item, spider):
        self.items.append(ItemAdapter(item).asdict())

        if len(self.items) >= self.BATCH_SIZE:
            self.commit()

    def close_spider(self, spider):
        self.commit()

    def commit(self):
        host = os.getenv("ENGINE_HOST") or "http://localhost:3000"
        json = {"documents": self.items}
        resp = requests.post(host + "/api/docs", json=json)
        print(f"insert {resp.status_code} {resp.text}")

        self.items = []
