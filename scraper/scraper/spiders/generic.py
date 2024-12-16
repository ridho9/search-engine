from typing import Any
import urllib.parse
import scrapy
from bs4 import BeautifulSoup
from scrapy.crawler import Crawler
import scrapy.http
from scrapy.linkextractors import LinkExtractor  # type: ignore

import urllib
from scraper.items import PageItem, RawPageItem
from trafilatura import extract, extract_metadata


def clean_html_text(html_text):
    doc = BeautifulSoup(html_text, "lxml")
    text = doc.get_text(separator="\n")
    cleaned_text = "\n".join([x for x in text.splitlines() if x.strip() != ""])
    return cleaned_text


def get_netloc(url):
    return urllib.parse.urlparse(url).netloc


class GenericSpider(scrapy.Spider):
    name = "generic"

    link_extractor = LinkExtractor()

    def __init__(self, name: str | None = None, **kwargs: Any):
        super().__init__(name, **kwargs)

        self.start_urls = [kwargs["scrape_url"]]
        allowed = set()
        for url in self.start_urls:
            netloc = get_netloc(url)
            allowed.add(netloc)
            netloc_split = netloc.split(".")
            if netloc_split[0] == "www":
                allowed.add(".".join(netloc_split[1:]))

        self.allowed_domains = list(allowed)
        self.logger.info(f"{self.start_urls} {allowed} {self.allowed_domains}")

    @classmethod
    def from_crawler(cls, crawler: Crawler, *args: Any, **kwargs: Any):
        spider = super().from_crawler(crawler, *args, **kwargs)

        spider.settings.set("CLOSESPIDER_ITEMCOUNT", 10, priority="spider")
        spider.settings.set("LOG_LEVEL", "INFO", priority="spider")

        scrape_url = kwargs["scrape_url"]
        domain = get_netloc(scrape_url)

        # FEEDS = {f"output/{domain}.jsonl": {"format": "jsonlines", "overwrite": True}}
        # spider.settings.set("FEEDS", FEEDS, priority="spider")

        return spider

    def parse(self, response):
        self.logger.info(f"Parse {response.url}")
        yield RawPageItem(url=response.url, raw_html=response.text)

        # links = response.css("a::attr(href)").getall()
        links = self.link_extractor.extract_links(response=response)
        yield from response.follow_all(links, callback=self.parse)
