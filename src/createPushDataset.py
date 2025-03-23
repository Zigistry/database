import json
import datasets
import os

JSON_FILES = ("games.json", "programs.json", "gui.json", "main.json", "web.json")

USER_AUTH_TOKEN = os.getenv("HF_AUTH_TOKEN")


def createDataset():
    """
    Create a dataset from the JSON files in the src directory
    """
    dataset = []
    for file in JSON_FILES:
        with open("./jsons/"+file, "r") as f:
            data = json.load(f)
            dataset += data
    return dataset


def pushDataset():
    """
    Push the dataset to a file
    """
    dataset = createDataset()

    hfDataset = datasets.Dataset.from_list(dataset)
    hfDataset.push_to_hub("zigistry/zigistry-complete-dataset", token=USER_AUTH_TOKEN)


if __name__ == "__main__":
    pushDataset()
