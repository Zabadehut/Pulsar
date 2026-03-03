use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Audience {
    Beginner,
    Expert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Fr,
    En,
}

impl Locale {
    pub fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "en" | "en-us" | "en-gb" => Self::En,
            _ => Self::Fr,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReferenceText {
    pub title: &'static str,
    pub summary: &'static str,
    pub beginner: &'static str,
    pub expert: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ReferenceEntry {
    pub id: &'static str,
    pub panel: &'static str,
    pub audience: Audience,
    pub aliases: &'static [&'static str],
    pub tags: &'static [&'static str],
    pub fr: ReferenceText,
    pub en: ReferenceText,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReferenceEntryView {
    pub id: &'static str,
    pub panel: &'static str,
    pub audience: Audience,
    pub title: &'static str,
    pub summary: &'static str,
    pub beginner: &'static str,
    pub expert: &'static str,
    pub aliases: &'static [&'static str],
    pub tags: &'static [&'static str],
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub score: usize,
    pub entry: ReferenceEntryView,
}

const CATALOG: &[ReferenceEntry] = &[
    ReferenceEntry {
        id: "cpu.usage",
        panel: "cpu",
        audience: Audience::Beginner,
        aliases: &["cpu", "usage", "global cpu", "processor"],
        tags: &["cpu", "usage", "global", "processor"],
        fr: ReferenceText {
            title: "CPU global",
            summary: "Montre la charge CPU totale observee sur l'hote.",
            beginner: "Plus la valeur se rapproche de 100%, plus le processeur est occupe.",
            expert: "Le pourcentage agrege additionne user, nice, system, irq, softirq et steal selon la source OS.",
        },
        en: ReferenceText {
            title: "Global CPU",
            summary: "Shows total CPU load observed on the host.",
            beginner: "The closer the value is to 100%, the busier the processor is.",
            expert: "The aggregate percentage combines user, nice, system, irq, softirq and steal depending on OS source data.",
        },
    },
    ReferenceEntry {
        id: "cpu.load",
        panel: "cpu",
        audience: Audience::Beginner,
        aliases: &["load", "load average", "1m", "5m", "15m"],
        tags: &["cpu", "load", "scheduler"],
        fr: ReferenceText {
            title: "Load average",
            summary: "Mesure la pression sur l'ordonnanceur sur 1, 5 et 15 minutes.",
            beginner: "Une load au-dessus du nombre de CPU peut signaler une file d'attente importante.",
            expert: "La semantique varie selon l'OS, mais reste utile comme signal de contention globale.",
        },
        en: ReferenceText {
            title: "Load average",
            summary: "Measures scheduler pressure over 1, 5 and 15 minutes.",
            beginner: "A load value above CPU count can indicate meaningful run queue pressure.",
            expert: "OS semantics differ, but it remains a useful host-level contention signal.",
        },
    },
    ReferenceEntry {
        id: "cpu.iowait",
        panel: "cpu",
        audience: Audience::Expert,
        aliases: &["iowait", "io wait", "cpu wait"],
        tags: &["cpu", "disk", "latency", "linux"],
        fr: ReferenceText {
            title: "CPU iowait",
            summary: "Temps passe par le CPU a attendre des IO blocantes.",
            beginner: "Une hausse d'iowait peut pointer vers un stockage lent ou sature.",
            expert: "Sur Linux, cette mesure vient des compteurs CPU et doit etre lue avec la latence disque et la queue depth.",
        },
        en: ReferenceText {
            title: "CPU iowait",
            summary: "CPU time spent waiting on blocking IO.",
            beginner: "Rising iowait can indicate slow or saturated storage.",
            expert: "On Linux this comes from CPU accounting and should be read alongside disk latency and queue depth.",
        },
    },
    ReferenceEntry {
        id: "memory.pressure",
        panel: "memory",
        audience: Audience::Beginner,
        aliases: &["memory pressure", "pressure", "available memory"],
        tags: &["memory", "pressure", "available", "swap"],
        fr: ReferenceText {
            title: "Pression memoire",
            summary: "Indice derive pour estimer la tension sur la memoire de l'hote.",
            beginner: "Une forte pression memoire signifie qu'il reste peu de marge avant swap ou reclaim agressif.",
            expert: "Pulsar derive ce score a partir de la memoire disponible, de l'usage et des compteurs associes.",
        },
        en: ReferenceText {
            title: "Memory pressure",
            summary: "Derived index estimating how stressed host memory is.",
            beginner: "High pressure means little margin remains before swap or aggressive reclaim.",
            expert: "Pulsar derives this score from available memory, usage and related counters.",
        },
    },
    ReferenceEntry {
        id: "memory.swap",
        panel: "memory",
        audience: Audience::Beginner,
        aliases: &["swap", "paging", "swpin", "swpout"],
        tags: &["memory", "swap", "paging"],
        fr: ReferenceText {
            title: "Swap",
            summary: "Montre l'utilisation de l'espace d'echange disque par la memoire virtuelle.",
            beginner: "Une forte activite swap peut ralentir fortement la machine.",
            expert: "Le swap doit etre interprete avec la pression memoire, pgin/pgout et les alertes.",
        },
        en: ReferenceText {
            title: "Swap",
            summary: "Shows disk-backed virtual memory usage.",
            beginner: "Heavy swap activity can slow the host down significantly.",
            expert: "Read swap together with memory pressure, pgin/pgout and alerts.",
        },
    },
    ReferenceEntry {
        id: "disk.await",
        panel: "disk",
        audience: Audience::Expert,
        aliases: &["await", "latency", "disk latency", "storage latency"],
        tags: &["disk", "await", "latency", "storage"],
        fr: ReferenceText {
            title: "Disk await",
            summary: "Latence moyenne observee par IO terminee.",
            beginner: "Plus cette valeur monte, plus les operations disque prennent du temps.",
            expert: "Pulsar la derive des compteurs de temps et d'IO completes, utile avec util% et queue depth.",
        },
        en: ReferenceText {
            title: "Disk await",
            summary: "Average latency observed per completed IO.",
            beginner: "Higher values mean disk operations take longer to finish.",
            expert: "Pulsar derives it from IO completion and timing counters; read it with util% and queue depth.",
        },
    },
    ReferenceEntry {
        id: "disk.queue_depth",
        panel: "disk",
        audience: Audience::Expert,
        aliases: &["queue depth", "qd", "io queue"],
        tags: &["disk", "queue", "latency", "saturation"],
        fr: ReferenceText {
            title: "Queue depth",
            summary: "Approximation du nombre moyen d'IO en attente ou en cours.",
            beginner: "Une queue depth qui monte avec la latence indique souvent une saturation.",
            expert: "Cette valeur vient du temps IO pondere, donc elle reste une approximation aggregate par device.",
        },
        en: ReferenceText {
            title: "Queue depth",
            summary: "Approximation of the average number of pending or active IOs.",
            beginner: "If queue depth rises with latency, storage saturation is likely.",
            expert: "This comes from weighted IO time, so it remains an aggregated per-device approximation.",
        },
    },
    ReferenceEntry {
        id: "network.tcp",
        panel: "network",
        audience: Audience::Beginner,
        aliases: &["tcp", "connections", "established", "listen", "time_wait"],
        tags: &["network", "tcp", "connections", "listen"],
        fr: ReferenceText {
            title: "Connexions TCP",
            summary: "Resume l'etat courant des connexions reseau TCP.",
            beginner: "Established montre les connexions actives, Listen les sockets en attente, TimeWait les fins de session recentes.",
            expert: "Une hausse anormale de TimeWait, retrans ou syn peut signaler un probleme applicatif ou reseau.",
        },
        en: ReferenceText {
            title: "TCP connections",
            summary: "Summarizes the current state of TCP network connections.",
            beginner: "Established shows active sessions, Listen waiting sockets, TimeWait recent closed sessions.",
            expert: "Unusual rises in TimeWait, retrans or syn states can point to application or network issues.",
        },
    },
    ReferenceEntry {
        id: "network.retrans",
        panel: "network",
        audience: Audience::Expert,
        aliases: &["retrans", "retransmits", "packet loss"],
        tags: &["network", "retrans", "loss", "tcp"],
        fr: ReferenceText {
            title: "Retransmissions",
            summary: "Compteur de segments TCP retransmis.",
            beginner: "Une hausse peut indiquer perte reseau, congestion ou cible lente.",
            expert: "A lire avec le debit, les erreurs, l'etat des connexions et la saturation applicative.",
        },
        en: ReferenceText {
            title: "Retransmissions",
            summary: "Counter of retransmitted TCP segments.",
            beginner: "Rising values can indicate packet loss, congestion or a slow peer.",
            expert: "Read together with throughput, errors, connection states and application saturation.",
        },
    },
    ReferenceEntry {
        id: "linux.psi",
        panel: "linux",
        audience: Audience::Expert,
        aliases: &["psi", "pressure stall", "stall", "linux pressure"],
        tags: &["linux", "psi", "pressure", "cpu", "memory", "io"],
        fr: ReferenceText {
            title: "PSI Linux",
            summary: "Pressure Stall Information mesure le temps perdu a cause de CPU, memoire ou IO.",
            beginner: "Si PSI monte, des taches restent bloquees faute de ressources.",
            expert: "Le avg10 est tres utile pour voir une degradation recente, surtout combine a cgroup et alerts.",
        },
        en: ReferenceText {
            title: "Linux PSI",
            summary: "Pressure Stall Information measures time lost to CPU, memory or IO pressure.",
            beginner: "When PSI rises, tasks are getting stalled by missing resources.",
            expert: "avg10 is especially useful for recent degradation, particularly with cgroup and alert context.",
        },
    },
    ReferenceEntry {
        id: "linux.cgroup",
        panel: "linux",
        audience: Audience::Expert,
        aliases: &["cgroup", "container", "cpu throttle", "memory max"],
        tags: &["linux", "cgroup", "container", "limits"],
        fr: ReferenceText {
            title: "Cgroup v2",
            summary: "Expose les limites et usages de ressources du groupe de controle courant.",
            beginner: "Pratique pour savoir si le processus tourne dans un conteneur ou sous quotas.",
            expert: "La memoire max, les pids et le throttling CPU aident a differencier un probleme host d'une limite imposee.",
        },
        en: ReferenceText {
            title: "Cgroup v2",
            summary: "Shows resource limits and usage for the current control group.",
            beginner: "Useful to tell whether the process runs inside a container or quota.",
            expert: "Memory max, pid limits and CPU throttling help separate host pressure from imposed limits.",
        },
    },
    ReferenceEntry {
        id: "process.cpu",
        panel: "process",
        audience: Audience::Beginner,
        aliases: &["process cpu", "top process", "pid", "threads"],
        tags: &["process", "cpu", "pid", "top"],
        fr: ReferenceText {
            title: "Top processus",
            summary: "Liste les processus les plus visibles selon CPU et autres compteurs.",
            beginner: "Commencez ici pour voir quel processus consomme CPU, memoire ou descripteurs.",
            expert: "La vue est utile pour un tri rapide, mais doit etre recroisee avec watch, snapshot ou replay.",
        },
        en: ReferenceText {
            title: "Top processes",
            summary: "Lists the most visible processes by CPU and related counters.",
            beginner: "Start here to see which process is using CPU, memory or file descriptors.",
            expert: "This is a fast triage view and should be cross-checked with watch, snapshot or replay.",
        },
    },
    ReferenceEntry {
        id: "process.jvm",
        panel: "process",
        audience: Audience::Expert,
        aliases: &["jvm", "java", "jvm detection"],
        tags: &["process", "jvm", "java"],
        fr: ReferenceText {
            title: "Detection JVM",
            summary: "Marque certains processus comme JVM selon des heuristiques simples.",
            beginner: "Le tag JVM aide a reperer rapidement une application Java dans la liste.",
            expert: "Ce n'est pas encore une detection profonde de runtime; le signal reste heuristique.",
        },
        en: ReferenceText {
            title: "JVM detection",
            summary: "Marks some processes as JVMs using simple heuristics.",
            beginner: "The JVM tag helps spot Java applications quickly in the process list.",
            expert: "This is not deep runtime detection yet; the signal is still heuristic.",
        },
    },
    ReferenceEntry {
        id: "alerts",
        panel: "alerts",
        audience: Audience::Beginner,
        aliases: &["alerts", "warning", "critical", "health"],
        tags: &["alerts", "health", "thresholds"],
        fr: ReferenceText {
            title: "Alertes",
            summary: "Les alertes synthetisent les signaux les plus urgents du snapshot.",
            beginner: "Utilisez-les comme point d'entree, puis remontez vers CPU, memoire, disque ou reseau.",
            expert: "Les alertes actuelles sont locales et basees sur seuils; elles donnent du contexte mais pas une RCA complete.",
        },
        en: ReferenceText {
            title: "Alerts",
            summary: "Alerts summarize the most urgent signals in the current snapshot.",
            beginner: "Use them as an entry point, then drill into CPU, memory, disk or network.",
            expert: "Current alerts are local and threshold-based; they provide context, not full RCA.",
        },
    },
];

pub fn catalog_views(locale: Locale) -> Vec<ReferenceEntryView> {
    CATALOG.iter().map(|entry| to_view(entry, locale)).collect()
}

pub fn search(query: &str, locale: Locale) -> Vec<SearchHit> {
    let normalized = normalize(query);
    let mut hits: Vec<SearchHit> = CATALOG
        .iter()
        .filter_map(|entry| {
            score_entry(entry, &normalized).map(|score| SearchHit {
                score,
                entry: to_view(entry, locale),
            })
        })
        .collect();

    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.entry.title.cmp(b.entry.title))
    });
    hits
}

pub fn panel_matches_query(panel: &str, query: &str) -> bool {
    let normalized = normalize(query);
    if normalized.is_empty() {
        return false;
    }

    CATALOG
        .iter()
        .any(|entry| entry.panel == panel && score_entry(entry, &normalized).is_some())
}

fn to_view(entry: &ReferenceEntry, locale: Locale) -> ReferenceEntryView {
    let text = match locale {
        Locale::Fr => entry.fr,
        Locale::En => entry.en,
    };

    ReferenceEntryView {
        id: entry.id,
        panel: entry.panel,
        audience: entry.audience,
        title: text.title,
        summary: text.summary,
        beginner: text.beginner,
        expert: text.expert,
        aliases: entry.aliases,
        tags: entry.tags,
    }
}

fn score_entry(entry: &ReferenceEntry, query: &str) -> Option<usize> {
    if query.is_empty() {
        return Some(1);
    }

    let mut score = 0;
    for candidate in search_terms(entry) {
        let normalized = normalize(candidate);
        if normalized == query {
            score = score.max(100);
        } else if normalized.contains(query) {
            score = score.max(60);
        } else if query
            .split_whitespace()
            .all(|part| normalized.contains(part))
        {
            score = score.max(30);
        }
    }

    if score == 0 {
        None
    } else {
        Some(score)
    }
}

fn search_terms(entry: &ReferenceEntry) -> Vec<&'static str> {
    let mut terms = vec![
        entry.id,
        entry.panel,
        entry.fr.title,
        entry.fr.summary,
        entry.en.title,
        entry.en.summary,
    ];
    terms.extend_from_slice(entry.aliases);
    terms.extend_from_slice(entry.tags);
    terms
}

fn normalize(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_finds_alias_match() {
        let hits = search("latency", Locale::En);
        assert!(hits.iter().any(|hit| hit.entry.id == "disk.await"));
    }

    #[test]
    fn panel_query_match_is_detected() {
        assert!(panel_matches_query("memory", "swap"));
        assert!(!panel_matches_query("network", "swap"));
    }
}
