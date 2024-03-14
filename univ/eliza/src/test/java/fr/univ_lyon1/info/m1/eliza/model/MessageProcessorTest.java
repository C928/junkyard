package fr.univ_lyon1.info.m1.eliza.model;

import fr.univ_lyon1.info.m1.eliza.model.messageProcessor.MessageProcessor;
import org.junit.jupiter.api.Test;

import static org.hamcrest.Matchers.*;
import static org.hamcrest.MatcherAssert.assertThat;

/**
 * Tests for MessageProcessor.
 */
public class MessageProcessorTest {
    @Test
    void testFirstToSecondPerson() {
        // Given
        DialogState s = new DialogState();
        MessageProcessor p = new MessageProcessor(s);

        // Then
        assertThat(p.firstToSecondPerson("Je pense à mon chien."),
                is("vous pensez à votre chien."));

        assertThat(p.firstToSecondPerson("Je suis heureux."),
                is("vous êtes heureux."));

        assertThat(p.firstToSecondPerson("Je dis bonjour."),
                is("vous dites bonjour."));

        assertThat(p.firstToSecondPerson("Je vais à la mer."),
                is("vous allez à la mer."));

        assertThat(p.firstToSecondPerson("Je finis mon travail."),
                is("vous finissez votre travail."));

        assertThat(p.firstToSecondPerson("Je finis mon travail."),
                is("vous finissez votre travail."));
    }

    @Test
    void testCustomSentences() {
        // Given
        DialogState s = new DialogState();
        MessageProcessor p = new MessageProcessor(s);

        // Then
        assertThat(p.processMessage("Comment est votre blanquette ?"),
                is("Elle est bonne."));

        assertThat(p.processMessage("La terre est-elle plate ?"),
                is("Plate comme une pizza."));
    }

    @Test
    void testPickRandom() {

    }
}
