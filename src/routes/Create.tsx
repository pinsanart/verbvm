import { FlashcardInput } from "../components/FlashcardInput";

import { VocabularyFlashcard, PhoneticFlashcard, OrthographicFlashcard } from "../types/Flashcard";

export default function Create() {
  const vocabularyCard: VocabularyFlashcard = {
    id: "vocab-001",
    type: "vocabulary",
    front: {
      phrase: {
        content: "It's raining cats and dogs outside!"
      }
    },
    back: {
      wordOrExpression: {
        content: "raining cats and dogs"
      },
      meaning: {
        content: "Chover muito forte (chover canivetes)."
      }
    }
  };

  // 2. Exemplo de PhoneticFlashcard
  const phoneticCard: PhoneticFlashcard = {
    id: "phonetic-002",
    type: "phonetic",
    front: {
      phrase: {
        content: "I would like a cup of coffee."
      }
    },
    back: {
      ipaPhonetics: {
        content: "/aɪ wʊd laɪk ə kʌp əv ˈkɔfi/"
      }
    }
  };

  // 3. Exemplo de OrthographicFlashcard
  // (Note que este tipo não possui a propriedade 'back' na sua interface)
  const orthographicCard: OrthographicFlashcard = {
    id: "orthographic-003",
    type: "orthographic",
    front: {
      phrase: {
        content: "Amanhã, eu vou ao supermercado comprar pão."
        // Opcionalmente, você poderia adicionar um arquivo de áudio para ditado aqui
        // media: [{ id: "audio1", mimeType: "audio/mpeg", localPath: "/audio/dictation.mp3" }]
      }
    }
  };

  return (
    <div className="mx-auto flex w-full flex-col gap-8 p-6 md:max-w-[60%]">
      <h1 className="text-2xl font-semibold">Create</h1>

      <div className="flex flex-col gap-6">
        <FlashcardInput key={vocabularyCard.id} flashcard={vocabularyCard} />
        <FlashcardInput key={phoneticCard.id} flashcard={phoneticCard} />
        <FlashcardInput key={orthographicCard.id} flashcard={orthographicCard} />
      </div>
    </div>
  );
}
