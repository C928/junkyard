package fr.univ_lyon1.info.m1.eliza.model;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Tests for MessageData data wrapper class.
 */
public class MessageDataTest {
        @Test
        void testMessageDataMethods() {
            // Given
            MessageData d = new MessageData(true, "Some message", 0);

            // Then
            assertThat(d.isUser(), is(true));
            assertThat(d.getMessage(), is("Some message"));
            assertThat(d.getMsgNumber(), is(0));

            d.setMsgNumber(1);
            assertThat(d.getMsgNumber(), is(1));

            d.decrementMsgNumber();
            assertThat(d.getMsgNumber(), is(0));
        }

    }
