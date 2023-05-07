#TEMPORARY CODE - WILL BE REMOVED BY CPP VERSION
import ctranslate2
from typing import List


def translate_init(model1: str, device: str):
    translator = ctranslate2.Translator(model1, device=device)
    return translator


def translate_delete(t):
    del t


def output_to_str_arr(results) -> List[str]:
    return [x.hypotheses[0] for x in results]


def translate(translator, sources: List[List[str]]):
    return translator.translate_batch(source=sources)