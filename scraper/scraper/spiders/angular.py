import scrapy


class AngularSpider(scrapy.Spider):
    name = "angular"
    allowed_domains = ["angular.io"]
    start_urls = ["https://angular.io"]

    def parse(self, response):
        pass
