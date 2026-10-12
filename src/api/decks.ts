import { invoke } from "@tauri-apps/api/core";
import type { Deck, NewDeck } from "../types/Deck";

export const listDecks = () => invoke<Deck[]>("list_decks");

export const createDeck = (input: NewDeck) => invoke<Deck>("create_deck", { input });

export const deleteDeck = (name: string) => invoke<void>("delete_deck", { name });
