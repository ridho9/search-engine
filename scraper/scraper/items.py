# Define here the models for your scraped items
#
# See documentation in:
# https://docs.scrapy.org/en/latest/topics/items.html

import scrapy
from dataclasses import dataclass


# class PageItem(scrapy.Item):
#     # define the fields for your item here like:
#     # name = scrapy.Field()
#     url = scrapy.Field()
#     title = scrapy.Field()
#     text = scrapy.Field()
#     meta_title = scrapy.Field()
#     meta_desc = scrapy.Field()

#     def __repr__(self):
#         """only print out attr1 after exiting the Pipeline"""
#         return repr({"url": self.url, "title": self.title})


@dataclass
class RawPageItem:
    url: str
    raw_html: str

    def __repr__(self):
        return repr({"url": self.url})


@dataclass
class PageItem:
    url: str
    title: str
    text: str
    meta_desc: str
    meta_title: str

    def __repr__(self):
        return repr({"url": self.url, "title": self.title})
