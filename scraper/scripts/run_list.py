import urllib.parse
import requests
import urllib

seen_netloc = set()


def start_scrape(scrape_url):
    try:
        url = scrape_url.strip()
        if not url.startswith("https://"):
            url = "https://" + url

        resp = requests.get(url)
        final_url = resp.url

        netloc = urllib.parse.urlparse(final_url).netloc
        if netloc in seen_netloc:
            print(f"Skip {final_url} {netloc=} as it is a duplicate")
        seen_netloc.add(netloc)

        headers = {
            "Content-Type": "application/x-www-form-urlencoded",
        }

        data = {
            "project": "myproject",
            "spider": "generic",
            "scrape_url": final_url,
            "jobid": netloc,
        }

        response = requests.post(
            "http://localhost:6800/schedule.json", headers=headers, data=data
        )
        resp = response.json()
        print(
            final_url,
            "\t",
            response.status_code,
            resp["status"],
            resp["jobid"],
            resp["node_name"],
        )
    except Exception as err:
        print("error", url, err)


with open("./domain_list.txt", "r") as f:
    for scrape_url in f.readlines():
        start_scrape(scrape_url)
