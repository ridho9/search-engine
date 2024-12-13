from scrapy.crawler import CrawlerProcess
from scraper.spiders.generic import GenericSpider
from scrapy.utils.project import get_project_settings
import requests


def start_scrape(scrape_url):
    process = CrawlerProcess(get_project_settings())
    url = scrape_url.strip()
    if not url.startswith("https://"):
        url = "https://" + url

    # try:
    resp = requests.get(url)
    final_url = resp.url

    print(scrape_url, url, final_url)

    process.crawl(GenericSpider, scrape_url=final_url)
    process.start()  # the script will block here until the crawling is finis
    # except Exception as err:
    #     print("Error", url, err)


with open("./domain_list_limit.txt", "r") as f:
    for scrape_url in f.readlines():
        start_scrape(scrape_url)

# start_scrape("api.drupal.org")
