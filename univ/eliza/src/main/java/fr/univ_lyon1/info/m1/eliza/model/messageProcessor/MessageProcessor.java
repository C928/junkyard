package fr.univ_lyon1.info.m1.eliza.model.messageProcessor;

import fr.univ_lyon1.info.m1.eliza.model.DialogState;

import java.util.List;
import java.util.Random;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Logic to process a message (and probably reply to it).
 */
public class MessageProcessor {
    private final Random random = new Random();
    private final DialogState dialogState;
    private final List<Verb> verbs = JsonLoader.loadJsonVerbFile();
    private final String[] randomResponses = JsonLoader.loadRandomResponses();

    /**
     * Constructeur de message processor.
     * @param dialogState Instance of dialog state to retrieve username variable.
     */
    public MessageProcessor(final DialogState dialogState) {
        this.dialogState = dialogState;
    }

    /**
     * Normalize the text: remove extra spaces, add a final dot if missing.
     * @param text to be normalized.
     * @return normalized text.
     */
    public String normalize(final String text) {
        return text.replaceAll("\\s+", " ")
                .replaceAll("^\\s+", "")
                .replaceAll("\\s+$", "")
                .replaceAll("[^\\.!?:]$", "$0.");
    }

    /**
     * Information about conjugation of a verb.
     */
    public static class Verb {
        private final String firstSingular;
        private final String secondPlural;

        public String getFirstSingular() {
            return firstSingular;
        }

        public String getSecondPlural() {
            return secondPlural;
        }

        Verb(final String firstSingular, final String secondPlural) {
            this.firstSingular = firstSingular;
            this.secondPlural = secondPlural;
        }
    }


    /**
     * Turn a 1st-person sentence (Je ...) into a plural 2nd person (Vous ...).
     * The result is not capitalized to allow forming a new sentence.
     * @param text The original sentence.
     * @return The 2nd-person sentence.
     */
    public String firstToSecondPerson(final String text) {
        String processedText = text
                .replaceAll("[Jj]e ([a-z]*)e ", "vous $1ez ");

        for (Verb v : verbs) {
            processedText = processedText.replaceAll(
                    "[Jj]e " + v.getFirstSingular(),
                    "vous " + v.getSecondPlural());
        }

        processedText = processedText
                .replaceAll("[Jj]e ([a-z]*)s ", "vous $1ssez ")
                .replace("mon ", "votre ")
                .replace("ma ", "votre ")
                .replace("mes ", "vos ")
                .replace("moi", "vous");

        return processedText;
    }

    /** Pick an element randomly in the array. */
    public <T> T pickRandom(final T[] array) {
        return array[random.nextInt(array.length)];
    }

    /**
     * Process the user message and generate an appropriate eliza response.
     * @param userMsg the message sent by user.
     * @return eliza response.
     */
    public String processMessage(final String userMsg) {
        String normalizedText = this.normalize(userMsg);

        Pattern pattern;
        Matcher matcher;

        pattern = Pattern.compile("Comment est votre blanquette \\?", Pattern.CASE_INSENSITIVE);
        matcher = pattern.matcher(normalizedText);
        if (matcher.matches()) {
            return "Elle est bonne.";
        }

        pattern = Pattern.compile("La terre est-elle plate \\?", Pattern.CASE_INSENSITIVE);
        matcher = pattern.matcher(normalizedText);
        if (matcher.matches()) {
            return "Plate comme une pizza.";
        }

        pattern = Pattern.compile("Par Osiris et par Apis, tu es un sanglier, un sanglier\\.\\.\\.",
                Pattern.CASE_INSENSITIVE);
        matcher = pattern.matcher(normalizedText);
        if (matcher.matches()) {
            return "Vous pouvez en allumez un seul à la fois ?";
        }

        if (random.nextBoolean()) {
            pattern = Pattern.compile("(.*)\\?");
            matcher = pattern.matcher(normalizedText);
            if (matcher.matches()) {
                return this.pickRandom(new String[] {
                        "Je vous renvoie la question.",
                        "Ici, c'est moi qui pose les questions.",
                });
            }
        }

        pattern = Pattern.compile(".*Je m'appelle (.*)\\.", Pattern.CASE_INSENSITIVE);
        matcher = pattern.matcher(normalizedText);
        if (matcher.matches()) {
            String newUserName = matcher.group(1);
            dialogState.setUserName(newUserName);
            return "Bonjour " + newUserName + ".";
        }

        String userName = dialogState.getUserName();
        if (normalizedText.equalsIgnoreCase("Au revoir.")) {
            if (random.nextBoolean()) {
                if (userName != null) {
                    return "Au revoir " + userName + ".";
                } else {
                    return "Au revoir.";
                }
            }

            return "Oh non, c'est trop triste de se quitter !";
        }

        pattern = Pattern.compile("Quel est mon nom \\?", Pattern.CASE_INSENSITIVE);
        matcher = pattern.matcher(normalizedText);
        if (matcher.matches()) {
            if (userName != null) {
                return "Votre nom est " + userName + ".";
            }

            return "Je ne connais pas votre nom.";
        }

        pattern = Pattern.compile("(Je .*)\\.", Pattern.CASE_INSENSITIVE);
        matcher = pattern.matcher(normalizedText);
        if (matcher.matches()) {
            final String startQuestion = this.pickRandom(new String[] {
                    "Pourquoi dites-vous que ",
                    "Pourquoi pensez-vous que ",
                    "Êtes-vous sûr que ",
            });
            return startQuestion + this.firstToSecondPerson(matcher.group(1)) + " ?";
        }

        // Nothing clever to say, answer randomly
        if (random.nextBoolean() || random.nextBoolean()) {
            return this.pickRandom(randomResponses);
        }

        // Default answer
        if (userName != null) {
            return "Qu'est-ce qui vous fait dire cela, " + userName + " ?";
        }

        return "Qu'est-ce qui vous fait dire cela ?";
    }
}
