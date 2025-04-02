import json
from datasets import Dataset
from libs import constants


def pushDataset():
    """
    Push the dataset to a file
    """
    dataset = json.load(open(constants.PROGRAMS_JSON_FILES[0], "r"))

    hfDataset = Dataset.from_list(dataset)
    hfDataset.push_to_hub("zigistry/programs", token=constants.HUGGING_FACE_API_KEY)


if __name__ == "__main__":
    pushDataset()
