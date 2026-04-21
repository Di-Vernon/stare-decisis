//! Claude Squad 연동 지점. Day-1은 세션 이름 규약뿐.

pub fn squad_session_name(task_id: &str) -> String {
    format!("myth-task-{task_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_format() {
        assert_eq!(squad_session_name("T1.1"), "myth-task-T1.1");
    }
}
