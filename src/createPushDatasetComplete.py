import json
import datasets
import os

JSON_FILES = ("main.json", "web.json", "games.json", "gui.json")

USER_AUTH_TOKEN = os.getenv("HF_AUTH_TOKEN")


def createDataset():
    """
    Create a dataset from the JSON files in the src directory
    """
    dataset = []
    seen = set()
    for file in JSON_FILES:
        with open("./jsons/" + file, "r") as f:
            data = json.load(f)
            for item in data:
                item_str = json.dumps(item, sort_keys=True)
                if item_str not in seen:
                    seen.add(item_str)
                    dataset.append(item)
    return dataset


def pushDataset():
    """
    Push the dataset to a file
    """
    dataset = createDataset()

    hfDataset = datasets.Dataset.from_list(dataset)
    hfDataset.push_to_hub("zigistry/packages", token=USER_AUTH_TOKEN)


if __name__ == "__main__":
    pushDataset()
