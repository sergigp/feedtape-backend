pub fn get_voice_id(voice_name: &str) -> String {
    match voice_name {
        "Lucia" => "voice_lucia_es",
        "Sergio" => "voice_sergio_es",
        "Conchita" => "voice_conchita_es",
        "Matthew" => "voice_matthew_en",
        "Joanna" => "voice_joanna_en",
        "Amy" => "voice_amy_en",
        "Celine" => "voice_celine_fr",
        "Mathieu" => "voice_mathieu_fr",
        "Hans" => "voice_hans_de",
        "Marlene" => "voice_marlene_de",
        "Ricardo" => "voice_ricardo_pt",
        "Ines" => "voice_ines_pt",
        "Carla" => "voice_carla_it",
        "Giorgio" => "voice_giorgio_it",
        _ => "voice_lucia_es",
    }
    .to_string()
}
