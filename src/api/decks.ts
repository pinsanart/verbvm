import { invoke } from "@tauri-apps/api/core";
import type { Deck, NewDeck, UpdateDeck } from "../types/Deck";

export const listDecks = () => invoke<Deck[]>("list_decks");

export const createDeck = (input: NewDeck) => invoke<Deck>("create_deck", { input });

export const updateDeck = (name: string, input: UpdateDeck) => invoke<Deck>("update_deck", { name, input });

export const deleteDeck = (name: string) => invoke<void>("delete_deck", { name });
