from typing import Any
import urllib.parse
import scrapy
from bs4 import BeautifulSoup
from scrapy.crawler import Crawler
import scrapy.http
from scrapy.linkextractors import LinkExtractor  # type: ignore

import urllib
from scraper.items import PageItem


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
            allowed.add(get_netloc(url))

        self.allowed_domains = list(allowed)
        print(self.start_urls, allowed, self.allowed_domains)

    @classmethod
    def from_crawler(cls, crawler: Crawler, *args: Any, **kwargs: Any):
        spider = super().from_crawler(crawler, *args, **kwargs)

        spider.settings.set("CLOSESPIDER_ITEMCOUNT", 100, priority="spider")
        spider.settings.set("LOG_LEVEL", "INFO", priority="spider")

        scrape_url = kwargs["scrape_url"]
        domain = get_netloc(scrape_url)

        FEEDS = {f"output/{domain}.jsonl": {"format": "jsonlines", "overwrite": True}}
        spider.settings.set("FEEDS", FEEDS, priority="spider")

        return spider

    def parse(self, response):
        # return
        title = response.css("title::text").get()
        cleaned_text = clean_html_text(response.text)
        meta_title = response.css('meta[name="title"]::attr(content)').get()
        meta_desc = response.css('meta[name="description"]::attr(content)').get()

        self.logger.info(f"Parse {response.url} {title}")
        yield PageItem(
            url=response.url,
            title=title,
            text=cleaned_text,
            meta_title=meta_title,
            meta_desc=meta_desc,
        )

        # links = response.css("a::attr(href)").getall()
        links = self.link_extractor.extract_links(response=response)
        yield from response.follow_all(links, callback=self.parse)
