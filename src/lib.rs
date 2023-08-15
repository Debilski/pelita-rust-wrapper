
use pyo3::prelude::*;

use std::cell::OnceCell;
use std::collections::HashSet;

pub type Shape = (usize, usize);
pub type Pos = (usize, usize);


#[derive(FromPyObject)]
pub struct Bot {
    pub position: Pos,
    pub legal_positions: Vec<Pos>,
    pub walls: HashSet<Pos>,
    pub enemy: Vec<EnemyBot>,
    pub other: OtherBot,
    pub food: Vec<Pos>,
    pub shape: Shape,
    pub is_blue: bool,
    #[pyo3(from_py_with = "emptyonce")]
    pub _say: OnceCell<String>
}

fn emptyonce(_var: &PyAny) -> PyResult<OnceCell<String>> {
    Ok(OnceCell::new())
}

impl Bot {
    pub fn say(&self, text: &str) {
        self._say.get_or_init(|| {
            text.to_string()
        });
    }
}

#[derive(FromPyObject)]
pub struct OtherBot {
    pub position: Pos,
    pub legal_positions: Vec<Pos>,
    pub is_blue: bool,
}

#[derive(FromPyObject)]
pub struct EnemyBot {
    pub position: Pos,
    pub is_noisy: bool,
    pub legal_positions: Vec<Pos>,
    pub food: Vec<Pos>,
    pub is_blue: bool,
}

#[macro_export]
macro_rules! pelita_player {
    ($module_name: ident, $team_name: expr, $move_fn: expr) => {

        use pyo3::prelude::*;

        #[pyfunction]
        fn r#move(mut pybot: PyObject, state: PyObject) -> PyResult<Pos> {
            // ignoring the dict state that is passed from Pelita
            Python::with_gil(|py| {

                let rustbot: Result<Bot, _> = pybot.extract(py);
                match rustbot {
                    Ok(b) => {
                        let result = $move_fn(&b, state);

                        if let Some(text) = b._say.get() {
                            pybot.setattr(py, "_say", text);
                        }

                        Ok(result)
                    },
                    Err(error) => panic!("Problem: {:?}", error),
                }
            })
        }

        /// A Python module implemented in Rust.
        #[pymodule]
        fn $module_name(_py: Python, m: &PyModule) -> PyResult<()> {

            m.add("TEAM_NAME", $team_name)?;
            m.add_function(wrap_pyfunction!(r#move, m)?)?;
            Ok(())
        }
    };
}
