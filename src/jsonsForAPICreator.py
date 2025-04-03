import json

from libs import constants


def createPackagesDatasetForAPI():
    dataset = []
    seen = set()
    for file in constants.PACKAGES_JSON_FILES:
        with open("./jsons/" + file, "r") as f:
            data = json.load(f)
            for item in data:
                del item["readme_content"]
                item_str = json.dumps(item, sort_keys=True)
                if item_str not in seen:
                    seen.add(item_str)
                    dataset.append(item)
    json.dump(dataset, open("./jsonsForAPICompressed/packages.json", "w"))


def createProgramsDatasetForAPI():
    with open("./jsons/programs.json", "r") as f:
        data = json.load(f)
        for i in data:
            del i["readme_content"]
        json.dump(data, open("./jsonsForAPICompressed/programs.json", "w"))


if __name__ == "__main__":
    createPackagesDatasetForAPI()
    createProgramsDatasetForAPI()
