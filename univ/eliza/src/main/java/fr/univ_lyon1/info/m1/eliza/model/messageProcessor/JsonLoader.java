package fr.univ_lyon1.info.m1.eliza.model.messageProcessor;

import com.google.gson.Gson;
import com.google.gson.JsonObject;
import com.google.gson.stream.JsonReader;

import java.io.FileReader;
import java.io.Reader;
import java.net.URL;
import java.util.ArrayList;
import java.util.List;

/**
 * Load data used for eliza responses.
 */
public final class JsonLoader {
    /**
     * This class should not have a public default constructor.
     */
    private JsonLoader() {

    }

    /**
     * Load the list of 3rd group verbs and their correspondance from 1st person singular
     * (Je) to 2nd person plural (Vous). The file should be called verbs.json.
     * @return The list of verbs.
     */
    public static List<MessageProcessor.Verb> loadJsonVerbFile() {
        List<MessageProcessor.Verb> verbs = new ArrayList<>();
        URL verbsJson = JsonLoader.class.getResource("/response_data/verbs.json");
        if (verbsJson == null) {
            System.err.println("verbs.json does not exists");
            System.exit(1);
        }

        try (Reader reader = new FileReader(verbsJson.getFile());
             JsonReader jsonReader = new JsonReader(reader)) {
            Gson gson = new Gson();
            JsonObject jsonObject = gson.fromJson(jsonReader, JsonObject.class);

            for (String firstSingular : jsonObject.keySet()) {
                String secondPlural = jsonObject.get(firstSingular).getAsString();
                verbs.add(new MessageProcessor.Verb(firstSingular, secondPlural));
            }
        } catch (Exception e) {
            System.err.println("Could not load verbs.json" + e);
            System.exit(1);
        }

        return verbs;
    }

    /**
     * Load random sentence from random.json.
     * @return An array containing random sentences.
     */
    public static String[] loadRandomResponses() {
        String[] randomResponses = {};
        URL randomJson = JsonLoader.class.getResource("/response_data/random.json");
        if (randomJson == null) {
            System.err.println("random.json does not exists");
            System.exit(1);
        }

        try (Reader reader = new FileReader(randomJson.getFile())) {
            Gson gson = new Gson();
            randomResponses = gson.fromJson(reader, String[].class);

        } catch (Exception e) {
            System.err.println("Could not load random.json: " + e);
            System.exit(1);
        }

        return randomResponses;
    }

    /*
    private static Map<Pattern, String> readJsonToPatternMap(String jsonFilePath) {
        try (Reader reader = new FileReader(jsonFilePath)) {
            Gson gson = new GsonBuilder()
                    .registerTypeAdapter(Pattern.class, new PatternTypeAdapter())
                    .create();

            // Use TypeToken to capture generic type information
            return gson.fromJson(reader, new TypeToken<Map<Pattern, String>>() {}.getType());
        } catch (IOException e) {
            e.printStackTrace();
            return null; // Handle the exception as needed
        }
    }
     */
}
