<?php
//require_once("zapcallib.php");

const GROUPS_ENUM = [
    "12096", "12102", "12403", "12406", "12419",        // S1 G1A-1B-2A-2B-3A
    "12425", "12424", "12444", "12460", "12461",        // S1 G3B-4A-4B-5A-5B
    "51542", "51543", "51545", "51546", "51548",        // S2 G1A-1B-2A-2B-3A
    "51549", "51551", "51552", "51554","51555",         // S2 G3B-4A-4B-5A-5B
    "9311", "9312", "9313", "9314", "9292",             // S3 G1-2-3-4-6
    "9315", "9316", "9317", "9318", "9293", "5484",     // S4 G1-2-3-4-6A-6B
    "9272", "18783",                                    // ASPE1, ASPE2
    "9319", "9320",                                     // LP DEVOPS G1-2
    "5700",                                             // LP ESSIR
    "9321"                                              // LP SID
];

const LOCATIONS_ENUM = [
    "Amphi1", "Amphi2",                                         // Amphi
    "S10", "S11", "S12", "S15", "S26",                          // TD
    "S18 - TP Réseau", "S21", "S23 - TP Réseau",                // TP
    "S01", "S03", "S13", "S14", "S16", "S17", "S22", "S24",     // IT
    "026-Langues", "028", "040",                                // GEA
    "S04 - Petite salle de réunion", "S25-Salle de réunion",    // REU
    "H20", "H21",                                               // GC
    "S18 - TP Réseau", "S21", "S23 - TP Réseau",                // TP
    "S18 - TP Réseau", "S21", "S23 - TP Réseau",                // TP
    // "Préfa 1", "Préfa 2",                                       // Préfa
];

/*
 * Retrieve the ics file from the University server
 */
function parse_edt_url($group_id, $first_date, $last_date): string | int {
     foreach (GROUPS_ENUM as $group)
         if ($group_id === $group)
             return "https://adelb.univ-lyon1.fr/jsp/custom/modules/plannings/anonymous_cal.jsp"
                 ."?resources=$group_id&projectId=1&calType=ical&firstDate=$first_date&lastDate=$last_date";

     return 1;
}

/*
 * Parse description from the ics file which contains all the
 * professor names for an event
 */
function parse_description($description, $professor_name): int {
    $lines = explode("\n", $description);
    for ($i = 1; $i < sizeof($lines) -1; $i++)
        if (strcasecmp($lines[$i], $professor_name) == 0)
            return 1;

    return 0;
}

/*
 * Return all events for a specific professor between $first_date and $last_date
 */
function parse_edt_by_professor($professor_name, $first_date, $last_date): array | int {
    $date_str = "&projectId=1&calType=ical&firstDate=$first_date&lastDate=$last_date";

    $e_array = array();                 //used to store all events where $professor_name appears
    $uid_array = array();               //used to avoid recurring events (two groups have the same course)
    foreach (GROUPS_ENUM as $group) {
        $url = "https://adelb.univ-lyon1.fr/jsp/custom/modules/plannings/anonymous_cal.jsp"
            ."?resources=$group".$date_str;

        $ical_content = file_get_contents($url);
        if(!$ical_content)
            return 1;

        try {
            $ical = new ZCiCal($ical_content);
        } catch(Exception $e) {
            return 1;
        }

        foreach ($ical->tree->child as $event) {
            if ($event->getName() == "VEVENT") {
                $check = 0;
                foreach ($event->data as $key => $value) {
                    if($key == "DESCRIPTION")
                        if (parse_description($value->getValues(), $professor_name))
                            $check = 1;

                    $break = 0;
                    if($key == "UID" && $check == 1) {
                        $uid = $value->getValues();
                        foreach ($uid_array as $stored_uid)
                            if ($uid == $stored_uid) {
                                $break = 1;
                                break;
                            }

                        if (!$break) {
                            $uid_array[] = $uid;
                            $e_array[] = $event;
                        }
                    }
                }
            }
        }
    }

    if(sizeof($uid_array) != sizeof($e_array))
        return 1;

    return $e_array;
}

/*
 * Return all events for a specific group between $first_date and $last_date
 */
function parse_edt_by_group($group_id, $first_date, $last_date): array | int
{
    $url = parse_edt_url($group_id, $first_date, $last_date);
    if ($url === 1)
        return 1;

    $e_array = array();
    $ical_content = file_get_contents($url);
    if(!$ical_content)
            return 1;

    try{
        $ical = new ZCiCal($ical_content);
    } catch(Exception $e) {
        return 1;
    }

    foreach ($ical->tree->child as $event)
        if ($event->getName() == "VEVENT")
            $e_array[] = $event;

    return $e_array;
}

/*
 * Create an array with an event each 2hours except for weekends
 * and Thursday afternoons
 */
function create_time_slots_array($first_date, $last_date): array {
    // $tmp1 = preg_split("-", $first_date);
    // $last_values = preg_split("-", $last_date);

    // $fmt_y1 = substr($first_date, 0, -6);
    // $fmt_m1 = substr($first_date, 5, -3);
    // $fmt_d1 = substr($first_date, 8);

    // $fmt_y2 = substr($last_date, 0, -6);
    // $fmt_m2 = substr($last_date, 5, -3);
    // $fmt_d2 = substr($last_date, 8);

    $time_slots = array();
    $d1 = date_create($first_date." 07:00:00");
    $d2 = date_create($last_date." 17:00:00");

    $second = 0;
    $swap = 1;
    $fm_first_date = "";
    while (1) {
        $d1_day = date('D', $d1->getTimestamp());
        $d1_hour = $d1->format('H');
        $d1_min = $d1->format('i');

        $fm_date = date_format($d1, "Ymd")."T".date_format($d1, "His")."Z";
        if ($second == 0) {
            $fm_first_date = $fm_date;
            $second = 1;
        }

        else {
            $time_slots[] = $fm_first_date."-".$fm_date;

            if ($swap == 1) {
                $fm_first_date = $fm_date;
                $swap = 0;
            }
            else {
                $second = 0;
                $swap = 1;
            }
        }

        switch ($d1_day) {
            case "Fri":
                if (($d1_hour == "16" && $d1_min >= "30") || $d1_hour > "16") {
                    date_add($d1, date_interval_create_from_date_string("3day"));
                    date_time_set($d1, 7, 00);
                }
                else if ($d1_hour == "11" || ($d1_hour == "12" && $d1_min < "30"))
                    date_time_set($d1, 12, 30);
                else
                    date_add($d1, date_interval_create_from_date_string("2hours"));
                break;

            case "Thu":
                if ($d1_hour >= "11") {
                    date_add($d1, date_interval_create_from_date_string("1day"));
                    date_time_set($d1, 7, 00);
                }
                else
                    date_add($d1, date_interval_create_from_date_string("2hours"));
                break;

            case "Sat":
                date_add($d1, date_interval_create_from_date_string("2days"));
                date_time_set($d1, 7, 00);
                break;

            case "Sun":
                date_add($d1, date_interval_create_from_date_string("1day"));
                date_time_set($d1, 7, 00);
                break;

            default:
                if ($d1_hour >= "17") {
                    date_add($d1, date_interval_create_from_date_string("1day"));
                    date_time_set($d1, 7, 00);
                }
                else if ($d1_hour >= "11" && $d1_hour < "13")
                    date_time_set($d1, 13, 00);
                else
                    date_add($d1, date_interval_create_from_date_string("2hours"));
        }
        if ($d1 > $d2)
            break;
    }

    return $time_slots;
}

/*
 * Remove the taken slots for a specific professor and group
 */
function remove_used_time_slots($events_array, $time_slots) {
    foreach ($events_array as $events) {
        $index = 0;
        foreach ($events as $event) {
            $d_start = "";
            $d_end = "";
            foreach ($event->data as $key => $value) {
                if ($key == "DTSTART") {
                    $d_start = $value->getValues();
                    $index++;
                }

                if ($key == "DTEND") {
                    $d_end = $value->getValues();
                    $index--;
                }

                if ($index == 0) {
                    for ($y = 0; $y < sizeof($time_slots); $y++) {
                        $time_slots_arr = extract_date_from_hashmap($time_slots[$y]);
                        $f_slot = $time_slots_arr[0];
                        $l_slot = $time_slots_arr[1];
                        if (($f_slot >= $d_start && $l_slot <= $d_end) || $f_slot == $d_start || $l_slot == $d_end)
                            array_splice($time_slots, $y, 1);
                    }
                }
            }
        }
    }

    return $time_slots;
}

/*
 * Call functions and returns an array containing
 */
function get_common_unused_time_slots($professor_name, $group_id, $first_date, $last_date): array {
    $e_dates = create_time_slots_array($first_date, $last_date);

    $prof_events = parse_edt_by_professor($professor_name, $first_date, $last_date);
    $group_events = parse_edt_by_group($group_id, $first_date, $last_date);

    $events_array = [$group_events, $prof_events];
    return remove_used_time_slots($events_array, $e_dates);
}

/*
 * Return locations inside an array.
 * These are separated inside the ics file by ','
 */
function parse_location($locations): array {
    $occupied_locations = array();
    $lines = explode(",", $locations);
    foreach ($lines as $loc)
        foreach (LOCATIONS_ENUM as $loc_enum) {
            if ($loc == $loc_enum) {
                $occupied_locations[] = $loc;
                break;
            }
        }

    return $occupied_locations;
}

/*
 * YYYYMMDDTHHMMSSZ-YYYYMMDDTHHMMSSZ
 * < first_date > - < last_date  >
 */
function format_date_for_hashmap($first_date, $last_date): string {
    return $first_date.'-'.$last_date;
}

/*
 * YYYYMMDDTHHMMSSZ-YYYYMMDDTHHMMSSZ
 * < first_date > - < last_date  >
 */
function extract_date_from_hashmap($fm_date): array {
    return explode("-", $fm_date);
}

/*
 * Generate a "random" UID string to add it as
 * a node to the ZCiCal object.
 */
function generate_random_uid(): string {
    $random_uid = "";
    $chars = "0123456789";
    $chars_len = strlen($chars) - 1;
    for ($i = 0; $i < 54; $i++)
        $random_uid .= $chars[rand(0, $chars_len)];

    return $random_uid;
}

/*
 * Write a string representation of a ZCiCal object
 * inside an ics file.
 */
function write_ics_file($filename, $ical): int {
    $ical_file = fopen($filename, "w");
    if ($ical_file == 0)
        return 1;

    $ret = fwrite($ical_file, $ical->export());
    if ($ret == 0)
        return 1;

    fclose($ical_file);

    return 0;
}

/*
 * Create a hashmap to store all occupied locations with key -> datetime string
 * and value -> array of occupied locations for the specific datetime.
 * It then loops through the occupied locations and creates an event each 2 hours
 * containing all unoccupied locations for the specific 2hours interval.
 * After this process all events are written inside an ics file.
 */
function create_ics_file_to_move_a_course($professor_name, $group_id, $first_date, $last_date): int {
    $occupied_locations = array();
    $unused_time_slots = get_common_unused_time_slots($professor_name, $group_id, $first_date, $last_date);

    $date_str = "&projectId=1&calType=ical&firstDate=$first_date&lastDate=$last_date";
    foreach (GROUPS_ENUM as $group) {
        $ical_content = file_get_contents("https://adelb.univ-lyon1.fr"
                        ."/jsp/custom/modules/plannings/anonymous_cal.jsp?resources=$group".$date_str);

        try {
            $ical = new ZCiCal($ical_content);
        } catch(Exception) {
            return 1;
        }

        $end = 0;
        $fm_date = "";
        //stores occupied locations for each event interval
        foreach ($ical->tree->child as $event) {
            if ($event->getName() == "VEVENT") {
                $d_start = "";
                $d_end = "";
                foreach ($event->data as $key => $value) {
                    if ($key == "DTSTART")
                        $d_start = $value->getValues();

                    if ($key == "DTEND") {
                        $d_end = $value->getValues();
                        $end = 1;
                    }
                    if ($end == 1) {
                        $fm_date = format_date_for_hashmap($d_start, $d_end);
                    }

                    if ($key == "LOCATION") {
                        foreach (parse_location($value->getValues()) as $add_class) {
                            $found = 0;
                            if (isset($occupied_locations[$fm_date])) {
                                foreach ($occupied_locations[$fm_date] as $o_loc) {
                                    if ($o_loc == $add_class) {
                                        $found = 1;
                                        break;
                                    }
                                }
                            }
                            if ($found == 0) {
                                //add occupied classrooms for a specific event interval
                                $occupied_locations[$fm_date][] = $add_class;
                            }
                        }
                        $end = 0;
                    }
                }
            }
        }
    }

    $unoccupied_locations = array();
    foreach ($occupied_locations as $time_slot => $o_loc_array) {
        foreach (LOCATIONS_ENUM as $loc_enum) {
            $found = 0;
            foreach ($o_loc_array as $o_loc) {
                if ($loc_enum == $o_loc)
                {
                    $found = 1;
                    break;
                }
            }

            if ($found == 0) {
                //add unoccupied classrooms for a specific event interval
                $unoccupied_locations[$time_slot][] = $loc_enum;
            }
        }
    }

    //create ics file from $unoccupied_locations
    $ical = new ZCiCal();
    foreach ($unoccupied_locations as $date => $locations) {
        $found = 0;
        $fm_date = extract_date_from_hashmap($date);
        foreach ($unused_time_slots as $unused_time_slot) {
            $fm_slot = extract_date_from_hashmap($unused_time_slot);
            $f_slot = $fm_slot[0];
            $l_slot = $fm_slot[1];
            if (($fm_date[0] >= $f_slot && $fm_date[1] <= $l_slot) || $fm_date[0] == $f_slot && $fm_date[1] == $l_slot)
                $found = 1;
        }
        if ($found == 0)
            continue;

        $uloc_str = "";
        foreach ($locations as $uloc)
            $uloc_str .= $uloc.",";

        $uloc_str =  substr($uloc_str, 0, -1);

        $e_ical = new ZCiCalNode("VEVENT", $ical->curnode);
        // $e_ical->addNode(new ZCiCalDataNode("DTSTAMP:"));
        $e_ical->addNode(new ZCiCalDataNode("DTSTART:".$fm_date[0]));
        $e_ical->addNode(new ZCiCalDataNode("DTEND:".$fm_date[1]));
        $e_ical->addNode(new ZCiCalDataNode("SUMMARY:Salles Libres"));
        $e_ical->addNode(new ZCiCalDataNode("LOCATION:".$uloc_str));

        $currentDate = new DateTime();
        $date = $currentDate->format('Y/m/d H:i');
        $e_ical->addNode(new ZCiCalDataNode("DESCRIPTION:\\n\\nIUT\\n(Exported :".$date.")\\n"));
        $e_ical->addNode(new ZCiCalDataNode("UID:EDT".generate_random_uid()));
    }

    $ret = write_ics_file("ical.ics", $ical);
    if ($ret)
        return 1;

    return 0;
}

/*
 * Create a new ics file from an array of events
 */
function create_ics_from_event_array($events): int {
    $ical = new ZCiCal();
    foreach ($events as $event) {
        $e_ical = new ZCiCalNode("VEVENT", $ical->curnode);
        foreach ($event->data as $key => $value) {
            switch ($key) {
                case "DTSTAMP":
                    $e_ical->addNode(new ZCiCalDataNode("DTSTAMP:".$value->getValues()));
                    break;

                case "DTSTART":
                    $e_ical->addNode(new ZCiCalDataNode("DTSTART:".$value->getValues()));
                    break;

                case "DTEND":
                    $e_ical->addNode(new ZCiCalDataNode("DTEND:".$value->getValues()));
                    break;

                case "SUMMARY":
                    $e_ical->addNode(new ZCiCalDataNode("SUMMARY:".$value->getValues()));
                    break;

                case "LOCATION":
                    $e_ical->addNode(new ZCiCalDataNode("LOCATION:".$value->getValues()));
                    break;

                case "DESCRIPTION":
                    $e_ical->addNode(new ZCiCalDataNode("DESCRIPTION:".$value->getValues()));
                    break;

                case "UID":
                    $e_ical->addNode(new ZCiCalDataNode("UID:".$value->getValues()));
                    break;

                //case "CREATED":
                //    $e_ical->addNode(new ZCiCalDataNode("CREATED:".ZCiCal::fromSqlDateTime()));
            }
        }
    }

    $ret = write_ics_file("ical.ics", $ical);
    if ($ret)
        return 1;

    return 0;
}

/* Used for debugging */
function print_events($events) {
    foreach ($events as $event)
        foreach ($event->data as $key => $value) {
            if ($key == "SUMMARY" || $key == "DESCRIPTION")
                echo $value->getValues();

            else if ($key == "UID"|| $key == "DTSTART")
                echo $key.": ".$value->getValues()."\n";
        }
}

/*
$first_date = "2022-01-24";
$last_date = "2022-01-28";
$prof = "dupont jean";
$group_id = "9311"; //G1S3
*/

/*
$events = parse_edt_by_professor($prof, $first_date, $last_date);
if ($events === 1)
    exit("Error: Parsing calendar by professor failed\n");
// print_events($events);
*/

/*
$events = parse_edt_by_group($group_id, $first_date, $last_date);
if ($events === 1)
    exit("Error: Parsing calendar by group failed\n");

$ret = create_ics_from_event_array($events);
if ($ret === 1)
    exit("Error: Calendar could not be created\n");
*/

/*
$ret = create_ics_file_to_move_a_course($prof, "9311", $first_date, $last_date);
if ($ret == 1)
    exit("Error: Calendar from empty locations could not be created\n");
*/

?>
