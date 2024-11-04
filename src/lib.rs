
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFunction, PyString, PyTuple};

use std::cell::OnceCell;
use std::collections::HashSet;

pub type Shape = (usize, usize);
pub type Pos = (usize, usize);

#[derive(Debug)]
pub struct Layout {
    pub walls: HashSet<Pos>,
    pub food: Vec<Pos>,
    pub shape: Shape,
    pub bots: [Pos; 4],
}

#[derive(FromPyObject)]
pub struct Bot {
    pub position: Pos,
    #[pyo3(attribute("_initial_position"))]
    pub initial_position: Pos,
    pub legal_positions: Vec<Pos>,
    #[pyo3(from_py_with = "vec_to_hash_set")]
    pub walls: HashSet<Pos>,
    pub enemy: Vec<EnemyBot>,
    pub other: OtherBot,
    pub food: Vec<Pos>,
    pub shape: Shape,
    pub is_blue: bool,
    pub turn: usize,
    pub score: usize,
    pub round: usize,
    #[pyo3(from_py_with = "empty_once")]
    pub _say: OnceCell<String>
}

fn vec_to_hash_set(var: &Bound<PyAny>) -> PyResult<HashSet<Pos>> {
    let walls: Vec<Pos> = var.extract()?;
    Ok(walls.into_iter().collect())
}

fn empty_once(_var: &Bound<PyAny>) -> PyResult<OnceCell<String>> {
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
    #[pyo3(attribute("_initial_position"))]
    pub initial_position: Pos,
    pub legal_positions: Vec<Pos>,
    pub is_blue: bool,
    pub turn: usize,
    pub score: usize,
}

#[derive(FromPyObject)]
pub struct EnemyBot {
    pub position: Pos,
    #[pyo3(attribute("_initial_position"))]
    pub initial_position: Pos,
    pub is_noisy: bool,
    pub legal_positions: Vec<Pos>,
    pub food: Vec<Pos>,
    pub is_blue: bool,
    pub turn: usize,
    pub score: usize,
}

#[macro_export]
macro_rules! pelita_player {
    ($module_name: ident, $team_name: expr, $move_fn: expr, $state_t: ty) => {

        use pyo3::prelude::*;

        #[macro_use]
        extern crate lazy_static;

        #[pyfunction]
        fn r#move(mut pybot: PyObject, _state: PyObject) -> PyResult<Pos> {

            use std::sync::Mutex;
            lazy_static! {
                static ref state: Mutex<Option<$state_t>> = Mutex::new(None);
            }

            // ignoring the dict state that is passed from Pelita
            Python::with_gil(|py| {

                let rustbot: Result<Bot, _> = pybot.extract(py);
                let mut locked_state = state.lock().unwrap();
                match rustbot {
                    Ok(b) => {
                        let result = $move_fn(&b, &mut *locked_state);

                        if let Some(text) = b._say.get() {
                            pybot.setattr(py, "_say", text)?;
                        }

                        Ok(result)
                    },
                    Err(error) => panic!("Problem: {:?}", error),
                }
            })
        }

        /// A Python module implemented in Rust.
        #[pymodule]
        fn $module_name(m: &Bound<'_, PyModule>) -> PyResult<()> {

            m.add("TEAM_NAME", $team_name)?;
            m.add_function(wrap_pyfunction!(r#move, m)?)?;
            Ok(())
        }
    };
}

pub fn parse_layout(layout_str: &str) -> PyResult<Layout> {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let sys = PyModule::import_bound(py, "sys")?;
        let path = sys.getattr("path")?;
        path.call_method1("append", ("./venv/lib/python3.12/site-packages",))?;  // append my venv path

        let pelita = PyModule::import_bound(py, "pelita")?;
        let layout_dict: Bound<'_, PyDict> = pelita.getattr("layout")?.call_method1("parse_layout", (layout_str,))?.extract()?;

        let walls = vec_to_hash_set(&layout_dict.get_item("walls")?.unwrap())?;
        let food = layout_dict.get_item("food")?.unwrap().extract()?;
        let shape = layout_dict.get_item("shape")?.unwrap().extract()?;
        let bots = layout_dict.get_item("bots")?.unwrap().extract()?;

        let layout = Layout {
            walls,
            food,
            shape,
            bots
        };
        Ok(layout)

    })
}

pub fn run_game(name: &str) -> PyResult<()> {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        run_game_(py, name)
    })
}

fn run_game_(py: Python<'_>, name: &str) -> PyResult<()> {
    use pyo3::types::PyList;
    use pyo3::types::PyString;
    use pyo3::types::PyFunction;
    use pyo3::types::IntoPyDict;

    let sys = PyModule::import_bound(py, "sys")?;
    let path = sys.getattr("path")?;
    path.call_method1("append", ("./venv/lib/python3.12/site-packages",))?;  // append my venv path

    let pelita = PyModule::import_bound(py, "pelita")?;
    let run_game = pelita.getattr("game")?.getattr("run_game")?;

    let layout_tuple: Bound<'_, PyTuple> = pelita.getattr("layout")?.call_method0("get_random_layout")?.extract()?; //(args.size, rng=rng, dead_ends=DEAD_ENDS);
    let layout_name = layout_tuple.get_item(0)?;
    let layout_string = layout_tuple.get_item(1)?;

    println!("Using layout '{}'", layout_name);

    let layout_dict = pelita.getattr("layout")?.call_method1("parse_layout", (layout_string,))?;

    let players_vec = vec![
        PyModule::import_bound(py, name)?.getattr("move")?,
        PyString::new_bound(py, "0").into_any(),
    ];

    let players: Bound<'_, PyList> = PyList::new_bound(py, players_vec);

    // let args = (players,);
    let kwargs = (vec![
        ("team_specs", players.into_any()),
        ("layout_dict", layout_dict)
        ]).into_py_dict_bound(py);
    run_game.call((), Some(&kwargs))?;
    Ok(())
}
