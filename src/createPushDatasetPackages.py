import json
from datasets import Dataset
from libs import constants


def createDataset():
    """
    Create a dataset from the JSON files in the src directory
    """
    dataset = []
    seen = set()
    for file in constants.PACKAGES_JSON_FILES:
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

    hfDataset = Dataset.from_list(dataset)
    hfDataset.push_to_hub("zigistry/packages", token=constants.HUGGING_FACE_API_KEY)


if __name__ == "__main__":
    pushDataset()
