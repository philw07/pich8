import subprocess
import json
import toml
import requests
import asyncio
from os import path
from bs4 import BeautifulSoup

OUT_FILE = 'NOTICE'
LICENSE_FILES = ['LICENSE', 'LICENSE.txt', 'LICENSE.md', 'LICENSE-MIT', 'LICENSE-MIT.md', 'LICENCE-MIT', 'LICENSE-APACHE', 'UNLICENSE']
HEADER = 'The source code of this package doesn\'t include any third party code,\nbut it depends on third party libraries which are statically linked into the resulting binary.\n\n'

class NoticeGenerator:
    def run(self):
        loop = asyncio.get_event_loop()
        loop.run_until_complete(self.main())
        loop.close()

    async def main(self):
        package_name = self.get_package_name()

        notices = HEADER
        metadata = subprocess.check_output('cargo metadata --format-version 1')
        packages = json.loads(metadata)['packages']

        tasks = []
        for pkg in packages:
            if pkg['name'] != package_name:
                tasks.append(asyncio.create_task(self.get_license(pkg)))

        results = await asyncio.gather(*tasks)
        for result in results:
            notices += result + '\n\n\n'

        with open(OUT_FILE, 'w', encoding='utf8') as f:
            f.write(notices.strip())


    def get_package_name(self):
        if not path.exists('Cargo.toml'):
            raise Exception('Cargo.toml not found')
        with open('Cargo.toml') as f:
            cargo_toml = f.read()
        t = toml.loads(cargo_toml)
        return t['package']['name']


    async def get_license(self, pkg):
        notice = ''

        # Local search
        pkg_path = path.dirname(pkg['manifest_path'])
        for license_file in LICENSE_FILES:
            license_path = path.join(pkg_path, license_file)
            if path.exists(license_path):
                with open(license_path, encoding='utf8') as f:
                    notice += f.read()

        # Remote search
        if len(notice) == 0:
            url = pkg['repository']
            if url and 'github' in url:
                resp = requests.get(url)
                if resp.status_code == 200:
                    soup = BeautifulSoup(resp.text, features='html.parser')
                    for license_file in LICENSE_FILES:
                        item = soup.select_one('a[title="{}"]'.format(license_file))
                        if item:
                            raw_url = 'https://raw.githubusercontent.com' + item['href'].replace('blob/', '')
                            resp = requests.get(raw_url)
                            if resp.status_code == 200:
                                notice = resp.text
                                break

            if len(notice) == 0:
                notice += 'Author(s): {}\nLicense(s): {}\n'.format(', '.join(pkg['authors']), pkg['license'])

        return '========== PACKAGE {0} START ==========\n\n{1}\n=========== PACKAGE {0} END ===========\n'.format(pkg['name'], notice)



if __name__ == '__main__':
    NoticeGenerator().run()
