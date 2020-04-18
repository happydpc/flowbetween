use super::*;

use std::sync::*;
use std::time::{Duration};

#[test]
fn collide_two_paths() {
    // Plus sign, combined into a path
    let edits = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AAiB+2FAAodjLHRF9PA8BAcNj5P1EA4AAAAAAAAAAAAAAGAAAAAAAAAAAAAACAAAAAAAAAAAAAADAAAAAAAAAAAAAANAAAAAAAAAAAAAAEAAAAAAAAlXAaIAEAAAAAAAA2MAsBACAAAAAAAAXPAGCACAAA8PAAArlAbEACAAAAAAAAGWAUCADAAAAAAAAbYA5BABAAAAAAAAVaArBABAAAAAAAAocAsBABAAAAAAAAieAQBABAAAAAAAAOgADBAAAAAAAAAA5hA1AAAAAAAAAAAXjAbAAAAAA4PAAAM3Cs9PAAAAAAAAAAkAU+PAAAA8PAAA8iAI+PAAAAAAAIAfhAU+PAAAAAAAAAyfAU+PAAAA4PAAADdAj+PAAAAAAAAADxA28P//PAAAAAAmvAv6P8/PA8PAAAQJAw+P0/PAAAAAA9GAi+P4/PA4PAAAbEA9+P3/PAAAAAAhCA9+Pw/PAAAAAA1AAL/Pn/PAAAAAAAAAl/PZ/PAAAAAA69PAAAF/PAAAAAA
        EB+Aj
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+FAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+DAjBAAoZmS0QAA4MzFIRt9PAsBAYNJBAB/PQAAAAAAAAAAAAAAIAAAAAAAAAAAAAADAAAAAAAAAAAAAABAAAAAAAAAAAAAACAAAAAAAAAAAAAALAAAAAAAAAAAAAAEAAAAAAAAoAAbkPDAAAAAAAANAAUyPCAAAAAAAAAAALvPCAAAAAAAAAAAKrPCAAAAAAEAi+P9mPCAAAAAAAAmzPkbOBAAAAAAAAU6PNYPDAAAAAAIA65PiWPAAAAAAAEAU6PhWPAAAAAAAAAm7PYXPAAAAAAAEAD9PEZPAAAAAAAAA9+PYbPAAAAAAAIAAAA6dPAAAAAAAAAsBA8CPAAAAAAAAAUCAflPAAAAAAAEAhCAAoPAAAAAAAAAGCAhqPAAAAAAAIAHCA3sPAAAAAAAAA4BAKvPAAAAAAAAAeBAexPAAAAAAAEAeBAYzPAAAAAAAAAQBAs1P//PAAAAAAXDA6tP+/PAAAAAADBAAAAu/PAAAAEAQBAhCA0/PAAAAAA2AAXDAq/PAAAAAAQBAGKAE/PAAAAAA
        EB+Dj
    ";

    // Run the edits
    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, edits);

    // Animation should contain a single layer and a frame with a single grouped item in it
    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    assert!(elements.len() == 1);

    let group = match elements[0] {
        Vector::Group(ref group)    => Some(group.clone()),
        _                           => None
    }.expect("Element should be a group");

    assert!(group.group_type() == GroupType::Added);
    assert!(group.elements().count() == 2);
}

#[test]
fn collide_three_paths_by_adding_to_existing_collision() {
    // Collide two paths then add an extra one to the collision
    let edits = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+FAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+DAoBAAoZmBwQAAoZGMIRa8PAACA4N8BAL/PRBAAAAAAAAAAAAAKAAAAAAAAAAAAAACAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAAAAAAAAAADAAAAAAAAAAAAAAEAAAAAAAAAAAAAAEAAAAAAAAaAAslPCAAAAAAAAAAAA0PCAAAAAAIAAAA5xPDAAAAAAAAI+PbEPDAAAAAAAAs9PslPKAAAAAAEA28P5hPCAAAAAAAAc8PwePBAAAAAAAAA8PAcPBAAAAAAEAm7PeZPBAAAAAAAAK7PzXPAAAAAAAAAL7PvWPAAAAAAAIAEtPJbNAAAAAAAAAO8PVaPAAAAAAAMAp8PpcPAAAAAAAAAD9PXfPAAAAAAAEAf9PviPAAAAAAAAAs9PslPAAAAAAAEA59P2oPAAAAAAAAAL7PsJPAAAAAAAIAw+PiyPAAAAAAAAAZ/P10PAAAAAAAEAl/P+2PAAAAAAAAAz/Po4P//PAAAAAAz/Pj6P//PAAAAAAAAAO8P+/PAAAAIAAAAr9P9/PAAAAAAAAAHGA8/PAAAAAAAAAuGAL/PAAAAAAAAAsFAf/PAAAAAAAAADNAD/PAAAAAA
        EB+Dj
        LBPtAAAAAA*+HIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+IAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+GAiBNnEAAomZyHRl9PAMCAAO6AAS9PNAAAAAAAAAAAAAAAAAAAAAAAAAAAAA9/PAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAeVA0/PGAAAAAAAApIAaAAOAAAAAAAATKA2AAFAAAAAAAApMADBAEAAA8PAAA8OArBACAAAAAAAACRA5BADAAAAAAAA8SAUCACAAAAAAAAoUAhCACAAA4PAAAStARFACAAAAAAAAoYAhCAEAAAAAAAAeZAGCACAAA8PAAAvaA5BABAAAAAAAAlbAeBAAAAAAAAAAOcAQBAAAAAAAAAApcA1AAAAAAAAAAA1cAbAAAAAA8PAAA1cAaAAAAAAAAAAARRBNAAAAAAAAAAAKXAAAAAAAA4PAAAoUA9+P+/PAAAAAA5RAU+P9/PAAAAAAKPAV+P9/PA8PAEACNAH+P8/PAAAAAA9KAU+P5/PAAAAAARJAV+P2/PAAAAAAXHAH+Pu/PAAAAAADFAi+Ph/PAAAAAAXDAI+P7+PAAAAAA
        EB+Gj
        LBPtAAAAAAS+JAhBAkDAAYzs6FRC9PAACAgO9AAf9PmAAAAAAAAAAAAAALAAAAAAAAAAAAAACAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAFAAAAAAAAAAAAAAEAAAAAAAAAAAAAADAAAAAAAAvOAHSADAAAAAAAA1IAhCAGAAAAAAAAAYARFACAAAAAAAAXPAuCAGAAAAAAAAeRA9CADAAAAAAAANUA9CADAAAAAAAAUWAJDADAAAAAAAApYAYDACAAA8PAAAvaAkDABAAAAAAAAU6AvGABAAAAAAAAXfA7CACAAA8PAAAOgAhCAAAAAAAAAADhAHCAAAAAAAAAArhAdBAAAAAAAAAARhAQBAAAAAAAAAAbgAoAAAAAA4PAAA8eAAAAAAAAAAAAAbcAAAAAAAAAAAAAzDBL/P//PAAAAAAeRAK/Pz/PA8PAAAlPAZ/P4/PAAAAAAHOAl/P3/PA4PAAA1MAAAAw/PAAAAAADJAAAAO/PAAAAAA
        EB+Jj
    ";

    // Run the edits
    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, edits);

    // Animation should contain a single layer and a frame with a single grouped item in it
    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    assert!(elements.len() == 1);

    let group = match elements[0] {
        Vector::Group(ref group)    => Some(group.clone()),
        _                           => None
    }.expect("Element should be a group");

    // Group should contain three elements
    assert!(group.group_type() == GroupType::Added);
    assert!(group.elements().count() == 3);

    // It should combine to be a single path
    let properties = frame.apply_properties_for_element(&elements[0], Arc::new(VectorProperties::default()));
    let group_path = group.to_path(&*properties, PathConversion::Fastest);

    assert!(group_path.unwrap().len() == 1);
}

#[test]
fn collide_three_paths_all_at_once() {
    // Draw two lines and join them to make an 'H' (which should all collide into one)
    let two_lines = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AAsBaqFAAoZOEIRN9PAMCAAONDA4AAcAAAAAAAAAAAAAAKAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAAAAAAAAAAFAAAAAAAAAAAAAAHAAAAAAAAAAAAAAFAAAAAAAAnAAUqPEAAAAAAAA9+PUmPCAAAAAAAA69PYvPEAAAAAAEAf9PEtPCAAAAAAAA28PiqPCAAAAAAAA38PcoPCAAAAAAAA28PHmPCAAAAAAAAD9PAkPCAAAAAAEAF9PiiPBAAAAAAAAU6PcAPAAAAAAAAAR9PhePBAAAAAAIAR9PRdPAAAAAAAAAS9P2cPAAAAAAAEAD9PbcPAAAAAAAAAD9PbcPAAAAAAAAAF9PDdPAAAAAAAAAD9P6dPAAAAAAAIAD9P9ePAAAAAAAAAv6PAEPAAAAAAAAAs9PElPAAAAAAAEA69PKnPAAAAAAAEAz7PzTPAAAAAAAAA69PbsPAAAAAAAIA59PVuPAAAAAAAAAV+PBwPAAAAAAAAAU+P5xP//PA8PAAA69PRpP//PAAAAEAl/PpwP9/PAAAAAAAAA96P4/PAAAAIAAAAp8P4/PAAAAAA1AAU+P0/PAAAAAA5BAAAAs/PAAAAAAsBAAAAa/PAAAAAAJDAeFA3+PAAAAAA
        EB+Aj
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+FAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+DAnBAAYzMgzQAAIAg0HR48PAACAEOYDAX+PlAAAAAAAAAAAAAALAAAAAAAAAAAAAACAAAAAAAAAAAAAADAAAAAAAAAAAAAAGAAAAAAAAAAAAAAKAAAAAAAAAAAAAAIAAAAAAAAAAAAAAFAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAAAAAAA//PAAAAAAoAAw+PAAAAAAAAANAAi6PDAAAAAAAAMAAB4PDAAAAAAAAAAAKnPDAAAAAAAAAAA8uPEAAAAAAAA9+PBsPBAAAAAAEAV+PRpPBAAAAAAAAf9P6lPBAAAAAAAA28P9iPCAAAAAAAAO8PlfPBAAAAAAIAz7P2cPBAAAAAAAAY3PyzOBAAAAAAEAm3P4xOBAAAAAAAAO8PiaPAAAAAAAIAc8PNcPAAAAAAAAAR9PhePAAAAAAAEAR9P3gPAAAAAAAAA69PXjPAAAAAAAEAH+P6lPAAAAAAAAAi6P4tOAAAAAAAAAL/PExP+/PAAAAIAL/PKzPAAAAAAAEAK/Pf1P//PAAAAAAZ/PJ3P//PAAAAAAl/Ps5P+/PAAAAAAm/P07P+/PAAAAIAl/PCRA8/PAAAAAA
        EB+Dj
    ";
    let join_lines = "
        LBPtAAAAAAS+GAiBYtCAA4QggGRZ8PAICAQOAAAOBATBAAAAAAAAAAAAALAAAAAAAAAAAAAAAAAAAAAAAAAAAAA+/PAAAAAAAAAAAAAAAAAAAAAAAAAAAHAAAAAAAAAAAAAAHAAAAAAAAbYAz/PGAAAAAAAA1IAAAADAAAAAAAAhKAAAAEAAAAAAAAoMAAAADAAAAAAAABcB5BACAAAAAAAANYANAALAAA4PAAAUaAAAABAAAAAAAAAcAAAAAAAAAAAAAsdAMAAAAAAAAAAAXfANAAAAAA8PAAAtxCAAAAAAAAAAAADlAAAABAAA8PAEAAkAAAAAAAAAAAAA5hAAAAAAAA4PAAAyfAAAAAAAAAAAAABoBL7PAAAAAAAAAEVAL/PAAAA8PAAAXTAY/PAAAAAAAAAfRAz/P//PAAAAAAyPAAAA//PAAAAAAHOAAAAAAAAAAAAAOMAAAA//PAAAAAAGKAbAA//PAAAAAAGWAeBA+/PAAAAAANAAAAAL/PAAAAAAAAAAAAW/PAAAAAA
        EB+Gj
    ";

    // The two lines on either side of the 'H'
    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, two_lines);

    // Animation should contain a single layer and a frame with a single grouped item in it
    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    // These don't join together
    assert!(elements.len() == 2);

    // The cross line that forms the 'H' shape
    perform_serialized_edits(&mut animation, join_lines);

    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    // Everything joined into one element now
    assert!(elements.len() == 1);

    let group = match elements[0] {
        Vector::Group(ref group)    => Some(group.clone()),
        _                           => None
    }.expect("Element should be a group");

    // Group should contain three elements. The original element is subsumed by the group and has no independent ID.
    assert!(group.group_type() == GroupType::Added);
    assert!(group.elements().count() == 3);
    assert!(group.elements().map(|elem| elem.id().id()).collect::<Vec<_>>() == vec![Some(0), Some(3), None]);

    // It should combine to be a single path
    let properties = frame.apply_properties_for_element(&elements[0], Arc::new(VectorProperties::default()));
    let group_path = group.to_path(&*properties, PathConversion::Fastest);

    assert!(group_path.unwrap().len() == 1);
}

#[test]
fn complex_path_collision() {
    // Draw a circle and a cross over the top. This generates a complex path with holes in it.
    let circle_with_cross = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        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BAAAAAAAAAAE8PAAAAAAAAAAAAm9PAAAAAAAAAAAAD8PAAAAAAAAAAAAm9PNBAAAAAAAAAAm9PAAAAAAAAAAAAD8PVBAAAAAAAAAAm9PAAAAAAAAAAAAm9PAAAAAAAAAAAAm9PNBAAAAAAAAAAm9PAAAAAAAAAAAAm9PAAAAAAAAAAAAm9PNBAAAAAAAAAA6+PAAAAAAAAAAAAm9PNBAAAAAAAAAAm9PNBAAAAAAAAAAm9PNBAAAAAAAAAAm9PNBAAAAAAAAAA6+PGBAAAAAAAAAAm9PNBAAAAAAAAAAm9PNBAAAAAAAAAAm9PAAAAAAAAAAAAm9PNBAAAAAAAAAA6+PGBAAAAAAAAAAm9PNBAAAAAAAAAAm9PNBAAAAAAAAAA6+PAAAAAAAAAAAAm9PNBAAAAAAAAAAm9PNBAAAAAAAAAA6+PAAAAAAAAAAAAm9PAAAAAAAAAAAA6+PGBAAAAAAAAAAm9PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAAm9PAAAAAAAAAAAAm9PAAAAAAAAAAAA6+PAAAAAAAAAAAAm9PAAAAAAAAAAAAm9PAAAAAAAAAAAAm9PAAAAAAAAAAAA7+PAAAAAAAAAAAAm9PAAAAAAAAAAAAm9PAAAAAAAAAAAAm9PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAAm9PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA++PAAAAAAAAAAAA6+PAAAAAAAAAAAA++PAAAAAAAAAAAA6+PAAAAAAAAAAAA++PAAAAAAAAAAAAA/PAAAAAAAAAAAAA/PAAAAAAAAAAAA++PAAAAAAAAAAAA++PAAAAAAAAAAAA/+PAAAAAAAAAAAA/+PAAAAAAAAAAAAAAAA/PAAAAAAAAA++PAAAAAAAAAAAAAAAD/PAAAAAAAAA
        EB+Aj
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+FAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+DA3CAAopJK0QAAo4+FIRA+PAAAAAAAAAD/PAAAAAAAAAAAAE8PAAAAAAAAAAAAU4PAAAAAAAAAAAAR4PAAAAAAAAAAAAD2PAAAAAAAAAVDAB2PAAAAAAAAAqBAB2PAAAAAAAAAjDAhzPAAAAAAAAAxBAhzPAAAAAAAAAxBAhzPAAAAAAAAAxBAhzPAAAAAAAAAxBAhzPAAAAAAAAA4BAzwPAAAAAAAAAxBAhzPAAAAAAAAAxBAhzPAAAAAAAAA4BAzwPAAAAAAAAAxBAhzPAAAAAAAAA4BAzwPAAAAAAAAAAAAzwPAAAAAAAAAxBAhzPAAAAAAAAA4BAzwPAAAAAAAAAAAAhzPAAAAAAAAAxBAhzPAAAAAAAAA4BAzwPAAAAAAAAAAAAhzPAAAAAAAAAuBA2zPAAAAAAAAAAAAR4PAAAAAAAAAICAqqPAAAAAAAAAAAAhzPAAAAAAAAAAAAhzPAAAAAAAAAxBAhzPAAAAAAAAAAAAhzPAAAAAAAAAAAAR4PAAAAAAAAAqBAB2PAAAAAAAAAAAAhzPAAAAAAAAABCA2tPAAAAAAAAAAAAhzPAAAAAAAAAAAAS6PAAAAAAAAAxBAhzPAAAAAAAAAAAAB2PAAAAAAAAAAAAhzPAAAAAAAAAAAAB2PAAAAAAAAAqBAB2PAAAAAAAAAAAAzwPAAAAAAAAAAAAS6PAAAAAAAAABCA2tPAAAAAAAAAAAAB2PAAAAAAAAAAAAhzPAAAAAAAAAxBAhzPAAAAAAAAAAAAS6PAAAAAAAAAAAAhzPAAAAAAAAAAAAB2PAAAAAAAAAAAAB2PAAAAAAAAAAAAR4PAAAAAAAAAAAAB2PAAAAAAAAAAAAS6PAAAAAAAAAAAAR4PAAAAAAAAAAAAS6PAAAAAAAAAAAAS4PAAAAAAAAAAAAS6PAAAAAAAAAAAAS6PAAAAAAAAAAAAS6PAAAAAAAAAAAAS6PAAAAAAAAAAAAS6PAAAAAAAAAAAAS6PAAAAAAAAAAAAE8PAAAAAAAAAAAAS6PAAAAAAAAAAAAE8PAAAAAAAAAAAAS6PAAAAAAAAAAAAE8PAAAAAAAAAAAAE8PAAAAAAAAAAAAn9PAAAAAAAAAAAAE8PAAAAAAAAAAAAn9PAAAAAAAAAAAAn9PAAAAAAAAAAAAE8PAAAAAAAAAAAAn9PAAAAAAAAAAAAn9PAAAAAAAAAAAAn9PAAAAAAAAAAAAn9PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAAn9PAAAAAAAAAAAA7+PAAAAAAAAAAAA7+PAAAAAAAAAAAAB/PAAAAAAAAA
        EB+Dj
        LBPtAAAAAAS+GA0Cj4EAAIkf8GRA+PAAAAAA9AAAAAAAAAAAAAA8DAAAAAAAAAAAAApHAAAAAAAAAAAAA8DAAAAAAAAAAAAAvHAAAAAAAAAAAAAvHAAAAAAAAAAAAAvHAAAAAAAAAAAAAfMAAAAAAAAAAAAA/JAAAAAAAAAAAAA/JAAAAAAAAAAAAAfMAAAAAAAAAAAAAKSAAAAAAAAAAAAANPAAAAAAAAAAAAAKSAAAAAAAAAAAAAWVAJCAAAAAAAAAAxYAgEAAAAAAAAAAKSAAAAAAAAAAAAAxYAgEAAAAAAAAAANPA5BAAAAAAAAAAKSACCAAAAAAAAAA/JArBAAAAAAAAAAxYAgEAAAAAAAAAAuFAcBAAAAAAAAAAKSACCAAAAAAAAAAfMAyBAAAAAAAAAANPA5BAAAAAAAAAANPAAAAAAAAAAAAAIPAAAAAAAAAAAAAKSACCAAAAAAAAAAKSAAAAAAAAAAAAAKSAAAAAAAAAAAAAKSAAAAAAAAAAAAAASAAAAAAAAAAAAAKSAAAAAAAAAAAAANPAAAAAAAAAAAAAKSAAAAAAAAAAAAAKSAAAAAAAAAAAAAKSA/9PAAAAAAAAAWVA49PAAAAAAAAAKSAAAAAAAAAAAAAWVA49PAAAAAAAAAWVAAAAAAAAAAAAAWVAAAAAAAAAAAAAWVAAAAAAAAAAAAAKSAAAAAAAAAAAAAWVAAAAAAAAAAAAAKSAAAAAAAAAAAAA/OAAAAAAAAAAAAAvHAAAAAAAAAAAAANPAAAAAAAAAAAAAeMAAAAAAAAAAAAAfMAAAAAAAAAAAAAfMAyBAAAAAAAAAAKSAAAAAAAAAAAAA/JAAAAAAAAAAAAAfMAAAAAAAAAAAAAfMAAAAAAAAAAAAAuFAAAAAAAAAAAAA/JAAAAAAAAAAAAA/JAAAAAAAAAAAAAvHAe+PAAAAAAAAAvHAe+PAAAAAAAAAvHAe+PAAAAAAAAAuFAl+PAAAAAAAAAuFAl+PAAAAAAAAA8DAs+PAAAAAAAAAuFAAAAAAAAAAAAA8DAs+PAAAAAAAAA6DAu+PAAAAAAAAA8DAs+PAAAAAAAAAZCA0+PAAAAAAAAA8DAs+PAAAAAAAAAZCAAAAAAAAAAAAAZCA0+PAAAAAAAAAZCA0+PAAAAAAAAAZCAAAAAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAFBAAAAAAAAAAAAAFBAAAAAAAAAAAAAFBAAAAAAAAAAAAA
        EB+Gj
    ";

    // The two lines on either side of the 'H'
    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, circle_with_cross);

    // Animation should contain a single layer and a frame with a single grouped item in it
    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    // These join into a single element
    assert!(elements.len() == 1);

    let group = match elements[0] {
        Vector::Group(ref group)    => Some(group.clone()),
        _                           => None
    }.expect("Element should be a group");

    // Group should contain three elements
    assert!(group.group_type() == GroupType::Added);
    assert!(group.elements().count() == 3);

    // It should combine to be a single path
    let properties = frame.apply_properties_for_element(&elements[0], Arc::new(VectorProperties::default()));
    let group_path = group.to_path(&*properties, PathConversion::Fastest);

    assert!(group_path.unwrap().len() == 1);
}

#[test]
fn draw_away_from_existing_group() {
    // Draw two plus signs then join them into a single path
    let plus_sign = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AArBmDGAAIAYHIRa9PAMCAEOtDA6/PTAAAAAAAAAAAAAAGAAAAAAAAAAAAAAEAAAAAAAAAAAAAAFAAAAAAAAAAAAAAQAAAAAAEAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAHCA9+PAAAAAAAAAnAAc8PCAAAAAAAAoAAL7PCAAAAAAAAQBAvyPCAAAAAAAANAAy3PDAAAAAAIAAAAj2PBAAAAAAAAAAAR1PBAAAAAAAAAAAzzPBAAAAAAAAAAAVyPCAAAAAAAAK/P2wPCAAAAAAAAw+PKvPBAAAAAAEAi+PetPBAAAAAAAAI6Pl/OAAAAAAAAAf9POoPBAAAAAAAAf9PAoPAAAAAAAAAD9PznPAAAAAAAAAR9PznPAAAAAAAIAF9PAoPAAAAAAAAAR9PboPAAAAAAAAAR9PpoPAAAAAAAAAs9PepPAAAAAAAEAO4PU2OAAAAAAAAAw+PHyPAAAAAAAEAw+PmzPAAAAAAAAAL/Pp0PAAAA8PAAAY/PH2PAAAAAAAAAm/PY3PAAAAAAAAAAAA9yPAAAAAAAAAAAAY7P+/PAAAAAAAAAD9P+/PAAAAAArBAU+P9/PAAAAAAQBAAAAx/PAAAAAAeBAKDAv/PAAAAAAQBAXDAa/PAAAAAAQBAGKAr+PAAAAAA
        EB+Aj
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+FAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+DAoB0LCAAoZ2RHR/8PAMCAQOIBAX9PpAAAAAAAAAAAAAAOAAAAAAAAAAAAAAFAAAAAAAAAAAAAACAAAAAAAAAAAAAADAAAAAAAAAAAAAAEAAAAAAAAAAAAAADAAAAAAAAAAAAAACAAAAAAAAAAAAAAKAAA8PAAAsZAs9PCAAAAAAAAlHA8+PCAAAAAAAAoIAL/PCAAAAAAAAGKAZ/PDAAAAAAAAs5AMAACAAAAAAAA8SA5BAGAAAAAAAAoUAhCABAAAAAAAAHWAvCAAAAAAAAAAJXAJDAAAAAAAAAAAYAKDAAAAA4PAAA2YA7CAAAAAAAAAA6xAEFAAAAAAAAAAMYA4BAAAAA8PAAAYXADBAAAAAAAAAAeVAoAAAAAAAAAAAzTAAAA//PAAAAAArRAAAA//PAAAAAAXPAY/P//PAAAAAAQNAi+P//PAAAAAAKLAI+P//PAAAAAAOQAz7P//PAAAAAAdFA69P8/PAAAAAANEAU+P9/PAAAAAA9CAU+P8/PAAAAAAGCAw+P5/PAAAAAA5BAw+P2/PAAAAAArBAL/Py/PAAAAAADBAl/Pn/PAAAAAA2AA0/PU/PAAAAAAAAAAAAP/PAAAAAA
        EB+Dj
    ";

    let line_away_from_plus_sign = "
        LBPtAAAAAAS+GAlBAAYzM00QAA4MTyFRG9PAMCAAOnDAB+PhAAAAAAAAAAAAAALAAAAAAAAAAAAAAFAAAAAAAAAAAAAAFAAAAAAAAAAAAAAHAAAAAAAAAAAAAAHAAAAAAAAAAAAAAFAAAAAAAAAAAAAADAAAAAAAAUCAm3PAAAAAAAAAAAAy3PFAAAAAAAAAAAe1PCAAAAAAEAAAA+yPCAAAAAAAAz/P2wPBAAAAAAAAw+PhuPBAAAAAAAAw+PBsPCAAAAAAEAp8P2QPBAAAAAAAAH+PDlPCAAAAAAAA69PYjPAAAAAAAIA69PUiPAAAAAAAAA59PiiPAAAAAAAAAI+PmjPAAAAAAAAAi+PflPAAAAAAAEA9+PNoPAAAAAAAAAz/P9qPAAAAAAAAAAAA5tPAAAAAAAAAAAAcwPAAAAAAAIAoAAznPAAAAAAAAA1AAL3PAAAAAAAAA1AAR5PAAAAAAAAA2AAL7P//PAAAAEADBAR9P+/PAAAAAA1AAi+P7/PAAAAAA5BAAAA5/PAAAAAAnAAXDAq/PAAAAAAAAARFAq/PAAAAAAAAApEAW+PAAAAAA
        EB+Gj
    ";

    // Draw a plus sign
    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, plus_sign);

    // Animation should contain a single layer and a frame with a two grouped items in it
    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    // Should join together to make a plus sign
    assert!(elements.len() == 1);

    let group = match elements[0] {
        Vector::Group(ref group)    => Some(group.clone()),
        _                           => None
    }.expect("Element should be a group");

    assert!(group.group_type() == GroupType::Added);
    assert!(group.elements().count() == 2);

    // Add an extra line away from the current group
    perform_serialized_edits(&mut animation, line_away_from_plus_sign);

    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    // This should remain as a separate element
    assert!(elements.len() == 2);
}

#[test]
fn join_two_groups() {
    // Draw two plus signs then join them into a single path
    let two_plus_signs = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AArBmDGAAIAYHIRa9PAMCAEOtDA6/PTAAAAAAAAAAAAAAGAAAAAAAAAAAAAAEAAAAAAAAAAAAAAFAAAAAAAAAAAAAAQAAAAAAEAAAAAAABAAAAAAAAAAAAAAAAAAAAAAAHCA9+PAAAAAAAAAnAAc8PCAAAAAAAAoAAL7PCAAAAAAAAQBAvyPCAAAAAAAANAAy3PDAAAAAAIAAAAj2PBAAAAAAAAAAAR1PBAAAAAAAAAAAzzPBAAAAAAAAAAAVyPCAAAAAAAAK/P2wPCAAAAAAAAw+PKvPBAAAAAAEAi+PetPBAAAAAAAAI6Pl/OAAAAAAAAAf9POoPBAAAAAAAAf9PAoPAAAAAAAAAD9PznPAAAAAAAAAR9PznPAAAAAAAIAF9PAoPAAAAAAAAAR9PboPAAAAAAAAAR9PpoPAAAAAAAAAs9PepPAAAAAAAEAO4PU2OAAAAAAAAAw+PHyPAAAAAAAEAw+PmzPAAAAAAAAAL/Pp0PAAAA8PAAAY/PH2PAAAAAAAAAm/PY3PAAAAAAAAAAAA9yPAAAAAAAAAAAAY7P+/PAAAAAAAAAD9P+/PAAAAAArBAU+P9/PAAAAAAQBAAAAx/PAAAAAAeBAKDAv/PAAAAAAQBAXDAa/PAAAAAAQBAGKAr+PAAAAAA
        EB+Aj
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+FAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+DAoB0LCAAoZ2RHR/8PAMCAQOIBAX9PpAAAAAAAAAAAAAAOAAAAAAAAAAAAAAFAAAAAAAAAAAAAACAAAAAAAAAAAAAADAAAAAAAAAAAAAAEAAAAAAAAAAAAAADAAAAAAAAAAAAAACAAAAAAAAAAAAAAKAAA8PAAAsZAs9PCAAAAAAAAlHA8+PCAAAAAAAAoIAL/PCAAAAAAAAGKAZ/PDAAAAAAAAs5AMAACAAAAAAAA8SA5BAGAAAAAAAAoUAhCABAAAAAAAAHWAvCAAAAAAAAAAJXAJDAAAAAAAAAAAYAKDAAAAA4PAAA2YA7CAAAAAAAAAA6xAEFAAAAAAAAAAMYA4BAAAAA8PAAAYXADBAAAAAAAAAAeVAoAAAAAAAAAAAzTAAAA//PAAAAAArRAAAA//PAAAAAAXPAY/P//PAAAAAAQNAi+P//PAAAAAAKLAI+P//PAAAAAAOQAz7P//PAAAAAAdFA69P8/PAAAAAANEAU+P9/PAAAAAA9CAU+P8/PAAAAAAGCAw+P5/PAAAAAA5BAw+P2/PAAAAAArBAL/Py/PAAAAAADBAl/Pn/PAAAAAA2AA0/PU/PAAAAAAAAAAAAP/PAAAAAA
        EB+Dj
        LBPtAAAAAAS+GAlBAAYzM00QAA4MTyFRG9PAMCAAOnDAB+PhAAAAAAAAAAAAAALAAAAAAAAAAAAAAFAAAAAAAAAAAAAAFAAAAAAAAAAAAAAHAAAAAAAAAAAAAAHAAAAAAAAAAAAAAFAAAAAAAAAAAAAADAAAAAAAAUCAm3PAAAAAAAAAAAAy3PFAAAAAAAAAAAe1PCAAAAAAEAAAA+yPCAAAAAAAAz/P2wPBAAAAAAAAw+PhuPBAAAAAAAAw+PBsPCAAAAAAEAp8P2QPBAAAAAAAAH+PDlPCAAAAAAAA69PYjPAAAAAAAIA69PUiPAAAAAAAAA59PiiPAAAAAAAAAI+PmjPAAAAAAAAAi+PflPAAAAAAAEA9+PNoPAAAAAAAAAz/P9qPAAAAAAAAAAAA5tPAAAAAAAAAAAAcwPAAAAAAAIAoAAznPAAAAAAAAA1AAL3PAAAAAAAAA1AAR5PAAAAAAAAA2AAL7P//PAAAAEADBAR9P+/PAAAAAA1AAi+P7/PAAAAAA5BAAAA5/PAAAAAAnAAXDAq/PAAAAAAAAARFAq/PAAAAAAAAApEAW+PAAAAAA
        EB+Gj
        LBPtAAAAAAS+HAkBAAomZmxQAA4MTzERU9PAMCAIO8AAk9PXAAAAAAAAAAAAAALAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADAAAAAAAAAAAAAAGAAAAAAAAAAAAAAHAAAAAAAAAAAAAAFAAAAAAAAAYAY/PEAAAAAAAAERA5BADAAAAAAAAhKA4BAEAAAAAAAAAMAUCACAAAAAAAAeNAhCACAAAAAAAA9OA9CABAAAAAAAAoQA9CABAAAAAAAAGSAJDACAAAAAAAAzTAYDABAAA8PAAAVCB5JAAAAAAAAAAAYAXDABAAA4PAAAaYAXDAAAAAAAAAA2YAKDAAAAAAAAAAoYA9CAAAAAAAAAABYAUCAAAAAAAAAAhWAQBAAAAAAAAAAoUAbAAAAAAAAAAAK7AX7PAAAAAAAIARJAs9P//PAAAAAAXHAs9PAAAAAAAAA4FAs9P//PAAAAAAbEAr9P//PAAAAAAkDA69P+/PAAAAAAoEAm7P+/PAAAAAAAAAY/Pw/PAAAAAAAAAAAAW/PAAAAAAR9PNAA++PAAAAAA
        EB+Hj
    ";

    let join_plus_signs = "
        LBPtAAAAAAS+IAvDasDAAYzsDIRB9PAICAIOCCAr8PqAAAAAAAAAAAAAANAAAAAAAAAAAAAAAAAAAAAAAAAAAAA//PAAAAAAAAAAAAAAAAAAAAAAAAAAAGAAAAAAAAAAAAAAHAAAAAAIAAAAAAAFAAAAAAAAeZAOkPKAAAAAAAArFA55PDAAAAAAAAGGAg5PCAAAAAAAAhGAf5PCAAAAAAAA9GA24PCAAAAAAAAXHAp4PCAAAAAAAA/PApwPBAAAAAAAAljANcPDAAAAAAAARJAt1PDAAA4PAAArJAR1PAAAAAAAAA6JAp0PAAAAAAAAAGKAA0PAAAA8PAAAUKAzzPAAAAAAAEAuKAYzPAAAAAAAAAKLAKzPAAAAAAAAA8WAHmPAAAA8PAAAAMALzPAAAAAAAAANMAlzPAAAAAAAAAOMAB0PAAAAAAAAANMAb0PAAAA4PAIAAMA20PAAAAAAAAAJXAiqPAAAAAAAAAXLAe1PAAAAAAAEAeVAiqPAAAAAAAAAsJAe1PAAAAAAAAADJAs1PAAAAAAAAANIA61PAAAAAAAAAyHAe1PAAAAAAAAAJHAf1PAAAAAAAAALnAbEPAAAAAAAEADJAKzPDAAAAAAAAdJAKzPAAAAAAAAArJAmzPAAAAAAAAAUKAlzPAAAAAAAAAUKAB0PAAAAAAAAAiKAO0PAAAA8PAAADVAEpPAAAAAAAIAGKAp0PAAAAAAAAA6JAp0PAAAAAAAAArJAb0PAAAAAAAAAeJA20PAAAAAAAEAQJAD1PAAAAAAAAADJAt1PAAAAAAAAADJAG2PAAAAAAAAADJAj2PAAAA4PAAA1IA92PAAAAAAAIA5ZAslPAAAAAAAAAaIAX3PBAAAAAAAAaIAm3PAAAAAAAAAAIAz3PAAAA8PAEAyHAy3PAAAAAAAAAXHAA4PAAAAAAAAA9GAO4PAAAAAAAAA1MASxPAAAAAAAAA6FA34PAAAAAAAIA4FAR5PAAAAAAAAA6FA34PAAAAAAAAA4FA24PAAAAAAAAA4FA34PAAAAAAAAA6FA24PAAAAAAAAA4FA34PAAAAAAAAA4FA34PAAAAAAAAA6FAR5PAAAAAAAAA4FAR5PAAAAAAAEA2QAHuPAAAAAAAAADFAV6PAAAAAAAAADFAi6PAAAAAAAAAoEAj6PAAAAAAAAAbEAv6PAAAAAAAAA/HA61PAAAAAAAAAyDAK7PAAAAAAAAAYDAL7PAAAAAAAAAXDA96PAAAAAAAAAkDAL7PAAAAAAAAAkDA96PAAAAAAAAAAEA96PAAAAAAAAANEA96PAAAAAAAAAbEA96PAAAAAAAAAoEAv6PAAAAAAAEACRABwPAAAAAAAAAfNAe1PAAAAAAAAA8GA96PAAAAAAAAA9GAY7PAAAAAAAIAvGAl7PAAAAAAAAAuGAz7PAAAA4PAAAUGAO8PAAAAAAAAA6FAO8PAAAAAAAAAdFAc8PAAAAAAAAA2EAp8PAAAAAAAAAMEA28PAAAAAAAAAKDAR9PAAAAAAAAAhCAs9PAAAAAAAEArBA69P//PAAAAAA2AAw+P//PAAAAAAaAA9+P+/PAAAAAAAAAl/P+/PAAAAAAAAAAAA9/PAAAAAAAAAAAA7/PAAAAAAm/PAAA2/PAAAAAAi+PNAAs/PA8PAAAw+PnAAF/PAAAAAAp8PbAAz+PAAAAAA
        EB+Ij
    ";

    // Draw two plus signs
    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, two_plus_signs);

    // Animation should contain a single layer and a frame with a two grouped items in it
    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    // The two plus signs should not join together but will each form their own group
    assert!(elements.len() == 2);

    // Join the two plus signs into a single grouped shape
    perform_serialized_edits(&mut animation, join_plus_signs);

    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    // Everything joined into one element now
    assert!(elements.len() == 1);

    let group = match elements[0] {
        Vector::Group(ref group)    => Some(group.clone()),
        _                           => None
    }.expect("Element should be a group");

    assert!(group.group_type() == GroupType::Added);
    assert!(group.elements().count() == 5);
}

#[test]
fn collide_with_paths_leaves_holes() {
    // This draws a partial circle, then a line to close it, then another line over the top, adding the results together
    // Encountered a bug where the initial circle is drawn fine but when over-drawn it removes the central 'hole', leaving
    // a solid object
    let path_circle = "
        +B
        LB+tAAAAAA
        LBPtAAAAAA*+BIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+CAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+AA0DAAYj6CzQAAom6RHRA+PAAAAAAE+PAAAAAAAAAAAAm9PAAAAAAAAAAAAQ4PAAAAAAAAAAAAA2PAAAAAAAAAAAAA2PAAAAAAAAAAAA1tPAAAAAAAAAAAA1wPAAAAAAAAAAAA1tP+7PAAAAAAAAA4tP+5PAAAAAAAAAsyPh2PAAAAAAAAAysPW1PAAAAAAAAA2vP71PAAAAAAAAAJvPXzPAAAAAAAAAm0Pn0PAAAAAAAAAM0PQyPAAAAAAAAA34PU1PAAAAAAAAAZ4PtyPAAAAAAAAAZ4PtyPAAAAAAAAAW+PH2PAAAAAAAAAd+PR4PAAAAAAAAAB8PGuPAAAAAAAAAAAAhzPAAAAAAAAAAAAhzPAAAAAAAAAAAAzwPAAAAAAAAAAAAnzPAAAAAAAAAAAAzwPAAAAAAAAABCA2tPAAAAAAAAAEIA2tPAAAAAAAAAqKAzsPAAAAAAAAADKA6vPAAAAAAAAAvMAAvPAAAAAAAAA8OA7uPAAAAAAAAA8OA7uPAAAAAAAAA6RAGuPAAAAAAAAAQUAPwPAAAAAAAAALTAPzPAAAAAAAAAhWAgyPAAAAAAAAAWVAe3PAAAAAAAAAMTAW1PAAAAAAAAAWVAn5PAAAAAAAAAWVAv7PAAAAAAAAAWVAv7PAAAAAAAAAMVAy7PAAAAAAAAAWVA49PAAAAAAAAAxYAx9PAAAAAAAAAQVAAAAAAAAAAAAAxYAAAAAAAAAAAAAWVAAAAAAAAAAAAAQVAAAAAAAAAAAAAKSAAAAAAAAAAAAA/RAAAAAAAAAAAAAWVAAAAAAAAAAAAAKSACCAAAAAAAAAANPAzDAAAAAAAAAAHSACEAAAAAAAAAAvHAjBAAAAAAAAAAwYABJAAAAAAAAAAuFAcBAAAAAAAAAAfMAXFAAAAAAAAAAaMAjDAAAAAAAAAAtHApEAAAAAAAAAA/JAAFAAAAAAAAAAvHAqEAAAAAAAAAAIGAoEAAAAAAAAAAMGAqEAAAAAAAAAAHGAnEAAAAAAAAAATEAUEAAAAAAAAAApEANGAAAAAAAAAAoCA9DAAAAAAAAAA2CAvFAAAAAAAAAAbBAvFAAAAAAAAAAiBAwHAAAAAAAAAAiBAvHAAAAAAAAAAAAAAKAAAAAAAAAAAAAgMAAAAAAAAAAAAA8OAAAAAAAAAAAAAgMAAAAAAAAAAJ+P5OAAAAAAAAAAN8POPAAAAAAAAAAp6PgMAAAAAAAAAAT6POPAAAAAAAAAAp6PgMAAAAAAAAAA58PwHAAAAAAAAAAB7P9JAAAAAAAAAAA7PAKAAAAAAAAAAA7PAKAAAAAAAAAA58PwHAAAAAAAAAAY7PJGAAAAAAAAAAJ9PvFAAAAAAAAAAJ9PvFAAAAAAAAAAX9P9DAAAAAAAAAAX9P9DAAAAAAAAAAX9P9DAAAAAAAAAAn9PZCAAAAAAAAAAX9P9DAAAAAAAAAAE8PoCAAAAAAAAAAm9PaCAAAAAAAAAAm9PaCAAAAAAAAAAm9PNBAAAAAAAAAA7+PFBAAAAAAAAAAm9PNBAAAAAAAAAA6+PGBAAAAAAAAAA6+PGBAAAAAAAAAAm9PNBAAAAAAAAAAm9PNBAAAAAAAAAAG8PAAAAAAAAAAAAm9PNBAAAAAAAAAAm9PNBAAAAAAAAAAn9PMBAAAAAAAAAAm9PNBAAAAAAAAAA6+PAAAAAAAAAAAAm9PNBAAAAAAAAAA6+PAAAAAAAAAAAA++PAAAAAAAAAAAAAAABBAAAAAAAAAA
        EB+Aj
        EB+Ap
        LBPtAAAAAA*+EIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+FAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+DAhJAAoGlQxQAAot9LHRA+PAAAAAA7BA8BAAAAAAAAAAZCANBAAAAAAAAAAZCAaCAAAAAAAAAAYCAAAAAAAAAAAAAZCANBAAAAAAAAAAYCAMBAAAAAAAAAAZCANBAAAAAAAAAA8DAVBAAAAAAAAAAZCANBAAAAAAAAAAZCANBAAAAAAAAAA8DAVBAAAAAAAAAAZCANBAAAAAAAAAA8DAVBAAAAAAAAAAZCAAAAAAAAAAAAA8DAVBAAAAAAAAAAXCAAAAAAAAAAAAAZCANBAAAAAAAAAAZCAAAAAAAAAAAAAYCAMBAAAAAAAAAAZCAAAAAAAAAAAAAFBAGBAAAAAAAAAAZCAAAAAAAAAAAAAZCANBAAAAAAAAAAFBAAAAAAAAAAAAAZCAAAAAAAAAAAAAFBAGBAAAAAAAAAAZCAAAAAAAAAAAAAYCAAAAAAAAAAAAAFBAGBAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAFBAAAAAAAAAAAAAZCAAAAAAAAAAAAAYCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCANBAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCANBAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCANBAAAAAAAAAAYCAAAAAAAAAAAAAZCANBAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAEBAFBAAAAAAAAAA8DAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAA8DAAAAAAAAAAAAAYCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAAZCAAAAAAAAAAAAA8DAAAAAAAAAAAAAZCA0+PAAAAAAAAAZCAAAAAAAAAAAAAFBA7+PAAAAAAAAAZCAAAAAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAZCA0+PAAAAAAAAAZCAAAAAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAXCA1+PAAAAAAAAAZCAAAAAAAAAAAAAFBA7+PAAAAAAAAAZCAAAAAAAAAAAAAZCA0+PAAAAAAAAAZCA0+PAAAAAAAAAZCAAAAAAAAAAAAAZCA0+PAAAAAAAAAFBA7+PAAAAAAAAAZCA0+PAAAAAAAAAZCAAAAAAAAAAAAAZCA0+PAAAAAAAAAYCA1+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAZCA0+PAAAAAAAAAZCAAAAAAAAAAAAAFBA7+PAAAAAAAAAZCAAAAAAAAAAAAAZCA0+PAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAEBA8+PAAAAAAAAAZCAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBA7+PAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAEBA8+PAAAAAAAAAZCAAAAAAAAAAAAAFBA7+PAAAAAAAAAYCAAAAAAAAAAAAAFBA7+PAAAAAAAAAZCAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAEBAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAEBAAAAAAAAAAAAAZCA0+PAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAZCAAAAAAAAAAAAAEBA8+PAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAYCAAAAAAAAAAAAAFBA7+PAAAAAAAAAZCAAAAAAAAAAAAAEBA8+PAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAYCA1+PAAAAAAAAAFBAAAAAAAAAAAAAYCA1+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAZCAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAZCAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAEBAAAAAAAAAAAAAZCAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAEBA8+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAEBAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAEBAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAABBA/+PAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAAAA7+PAAAAAAAAAFBA7+PAAAAAAAAAEBAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBA7+PAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAEBA8+PAAAAAAAAAZCA0+PAAAAAAAAAEBA8+PAAAAAAAAAZCA0+PAAAAAAAAAFBA7+PAAAAAAAAAFBA7+PAAAAAAAAAZCAAAAAAAAAAAAAFBA7+PAAAAAAAAAZCA0+PAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAEBA8+PAAAAAAAAAFBA7+PAAAAAAAAAEBAAAAAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAEBA8+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAYCAAAAAAAAAAAAAFBA7+PAAAAAAAAAZCA0+PAAAAAAAAAFBAAAAAAAAAAAAAZCA0+PAAAAAAAAAFBA7+PAAAAAAAAAYCAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAEBAAAAAAAAAAAAAAAA7+PAAAAAAAAAFBAAAAAAAAAAAAABBAAAAAAAAAAAAABBAAAAAAAAAAAAAAAA/+PAAAAAAAAA/AAAAAAAAAAAAAAABAA/PAAAAAAAAABBAAAAAAAAAAAAAFBA7+PAAAAAAAAABBA/+PAAAAAAAAAFBAAAAAAAAAAAAAEBA8+PAAAAAAAAAFBAAAAAAAAAAAAABBA/+PAAAAAAAAABBAAAAAAAAAAAAAFBA7+PAAAAAAAAABBA/+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAFBAAAAAAAAAAAAAAAA8+PAAAAAAAAAFBAAAAAAAAAAAAAAAA/+PAAAAAAAAAFBAAAAAAAAAAAAAAAAD/PAAAAAAAAAAAAD/PAAAAAAAAAABAA/PAAAAAAAAAAAA/+PAAAAAAAAAFBAAAAAAAAAAAAAAAA8+PAAAAAAAAAFBAAAAAAAAAAAAAAAA7+PAAAAAAAAAEBAAAAAAAAAAAAAAAA7+PAAAAAAAAAEBAAAAAAAAAAAAAAAA7+PAAAAAAAAAFBAAAAAAAAAAAAAAAA7+PAAAAAAAAAEBAAAAAAAAAAAAAAAA7+PAAAAAAAAAFBAAAAAAAAAAAAAFBA7+PAAAAAAAAAAAA7+PAAAAAAAAAFBA7+PAAAAAAAAABBA/+PAAAAAAAAAABAA/PAAAAAAAAAAAAA/PAAAAAAAAAFBAAAAAAAAAAAAABBA/+PAAAAAAAAABBA/+PAAAAAAAAABBAAAAAAAAAAAAAAAA7+PAAAAAAAAA/AAAAAAAAAAAAAA
        EB+Dj
        EB+Dp
    ";

    let overdraw_circle = "
        LBPtAAAAAA*+HIAAAAg+AAAAoABAAAICB+
        LBPtAAAAAAP+IAAAAoABAAAg/AHAAAAAAAAAyCBAAAAAAAAAg/A
        LBPtAAAAAAS+GAkEAAoJ6kzQAAopQSHRA+PAAAAAAB/PAAAAAAAAAAAA69PAAAAAAAAAAAA++PAAAAAAAAAAAAn9PAAAAAAAAAAAA++PAAAAAAAAAAAA19PAAAAAAAAAAAA/+PAAAAAAAAAAAA/+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA++PAAAAAAAAAAAA7+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA++PAAAAAAAAAAAA++PAAAAAAAAAAAA++PAAAAAAAAAAAA6+PAAAAAAAAAAAA++PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA++PAAAAAAAAAAAA++PAAAAAAAAAAAA/+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA7+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA++PAAAAAAAAAAAA++PAAAAAAAAAAAA/+PAAAAAAAAAAAA/+PAAAAAAAAAAAA++P/+PAAAAAAAAA6+PAAAAAAAAAAAA++P/+PAAAAAAAAA6+PAAAAAAAAAAAA++PAAAAAAAAAAAA6+P7+PAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAAm9PAAAAAAAAAAAA6+PAAAAAAAAAAAA++P/+PAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA++PAAAAAAAAAAAA/+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA7+P8+PAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+P7+PAAAAAAAAA++PAAAAAAAAAAAA6+PAAAAAAAAAAAA++PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+P7+PAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA7+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+P7+PAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA7+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+P7+PAAAAAAAAA++PAAAAAAAAAAAA/+PAAAAAAAAAAAAA/PAAAAAAAAAAAAAAAA/PAAAAAAAAA++PAAAAAAAAAAAA++PAAAAAAAAAAAAAAA7+PAAAAAAAAA6+PAAAAAAAAAAAA6+P7+PAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAAAAA7+PAAAAAAAAA6+PAAAAAAAAAAAA7+PAAAAAAAAAAAA6+P7+PAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+P7+PAAAAAAAAA7+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA/+PAAAAAAAAAAAA/+PAAAAAAAAAAAA++P/+PAAAAAAAAA++PAAAAAAAAAAAA6+PAAAAAAAAAAAA/+PAAAAAAAAAAAAAAA7+PAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAAn9PAAAAAAAAAAAA6+PAAAAAAAAAAAAm9PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA7+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA6+PGBAAAAAAAAAA6+PAAAAAAAAAAAA6+PAAAAAAAAAAAA++PAAAAAAAAAAAA++PAAAAAAAAAAAAA/PABAAAAAAAAAA
        EB+Gj
        EB+Gp
    ";

    // Draw two plus signs
    let mut animation = create_animation();
    perform_serialized_edits(&mut animation, path_circle);

    // At this point, the animation should contain a solid circle
    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    // Single element
    assert!(elements.len() == 1);

    // Should be a path with a hole
    let path = match &elements[0] {
        Vector::Path(path)  => path,
        _                   => { assert!(false); unimplemented!() }
    };

    // Two subpaths means it's a path with a hole
    assert!(path.path().to_subpaths().len() == 2);

    // Converting to path should be the same
    let properties      = frame.apply_properties_for_element(&elements[0], Arc::new(VectorProperties::default()));
    let to_path_fastest = path.to_path(&properties, PathConversion::Fastest).unwrap();
    let to_path_removed = path.to_path(&properties, PathConversion::RemoveInteriorPoints).unwrap();

    assert!(to_path_fastest.len() == 1);
    assert!(to_path_fastest[0].to_subpaths().len() == 2);

    assert!(to_path_removed.len() == 1);
    assert!(to_path_removed[0].to_subpaths().len() == 2);

    // Overdrawing the circle should still leave us with a hollow circle
    perform_serialized_edits(&mut animation, overdraw_circle);

    // At this point, the animation should still contain a solid circle
    let layer       = animation.get_layer_with_id(1).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    // Single element
    assert!(elements.len() == 1);

    // Should be a path with a hole
    let path = match &elements[0] {
        Vector::Path(path)  => path,
        _                   => { assert!(false); unimplemented!() }
    };
    assert!(path.path().to_subpaths().len() == 2);
}
