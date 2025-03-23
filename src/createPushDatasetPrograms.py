import json
import datasets
import os

JSON_FILE = "./jsons/programs.json"

USER_AUTH_TOKEN = os.getenv("HF_AUTH_TOKEN")



def pushDataset():
    """
    Push the dataset to a file
    """
    dataset = json.load(open(JSON_FILE, "r"))

    hfDataset = datasets.Dataset.from_list(dataset)
    hfDataset.push_to_hub("zigistry/programs", token=USER_AUTH_TOKEN)


if __name__ == "__main__":
    pushDataset()
