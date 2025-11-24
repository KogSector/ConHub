use crate::models::EntityFeatures;
use strsim::jaro_winkler;

pub fn calculate_email_match(f1: &EntityFeatures, f2: &EntityFeatures) -> Option<f32> {
    match (&f1.email, &f2.email) {
        (Some(e1), Some(e2)) if e1.to_lowercase() == e2.to_lowercase() => Some(1.0),
        _ => None,
    }
}

pub fn calculate_name_similarity(f1: &EntityFeatures, f2: &EntityFeatures) -> f32 {
    let name1 = f1.full_name.as_deref().or(f1.display_name.as_deref()).unwrap_or("");
    let name2 = f2.full_name.as_deref().or(f2.display_name.as_deref()).unwrap_or("");
    
    if name1.is_empty() || name2.is_empty() {
        return 0.0;
    }
    
    jaro_winkler(&name1.to_lowercase(), &name2.to_lowercase()) as f32
}

pub fn calculate_attribute_overlap(f1: &EntityFeatures, f2: &EntityFeatures) -> f32 {
    let mut matches = 0;
    let mut total = 0;

    if f1.username.is_some() || f2.username.is_some() {
        total += 1;
        if f1.username == f2.username && f1.username.is_some() {
            matches += 1;
        }
    }

    if f1.user_id.is_some() || f2.user_id.is_some() {
        total += 1;
        if f1.user_id == f2.user_id && f1.user_id.is_some() {
            matches += 1;
        }
    }

    if total > 0 {
        matches as f32 / total as f32
    } else {
        0.0
    }
}

pub fn calculate_graph_similarity(f1: &EntityFeatures, f2: &EntityFeatures) -> f32 {
    let repos1: std::collections::HashSet<_> = f1.associated_repositories.iter().collect();
    let repos2: std::collections::HashSet<_> = f2.associated_repositories.iter().collect();
    
    let intersection = repos1.intersection(&repos2).count();
    let union = repos1.union(&repos2).count();
    
    if union > 0 {
        intersection as f32 / union as f32
    } else {
        0.0
    }
}
