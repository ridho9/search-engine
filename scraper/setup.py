# Automatically created by: scrapyd-deploy

from setuptools import setup, find_packages

setup(
    name="project",
    version="1.0",
    packages=find_packages(),
    entry_points={"scrapy": ["settings = scraper.settings"]},
    zip_safe=False,
    strict_timestamps=False,
)
