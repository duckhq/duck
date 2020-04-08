use crate::builds::{Build, BuildStatus};
use crate::query;
use crate::query::{Constant, Expression, Operator, Property, Visitor};
use crate::DuckResult;

pub enum FilterResult {
    Retain,
    Filter,
    Error(String),
}

///////////////////////////////////////////////////////////
// Build filtering

pub struct BuildFilter {
    expression: Option<Expression>,
    evaluator: FilterEvaluator<Build>,
}

impl BuildFilter {
    pub fn new(expression: Option<String>) -> DuckResult<Self> {
        Ok(Self {
            expression: match expression {
                Some(expression) => {
                    let expression = query::parse(expression)?;
                    BuildFilterValidator::validate(&expression)?;
                    Some(expression)
                }
                None => None,
            },
            evaluator: FilterEvaluator::<Build>::new(),
        })
    }

    pub fn evaluate(&self, build: &Build) -> FilterResult {
        if let Some(expression) = &self.expression {
            match expression.accept(build, &self.evaluator) {
                Ok(result) => match result {
                    Constant::Boolean(r) => {
                        if r {
                            return FilterResult::Retain;
                        } else {
                            return FilterResult::Filter;
                        }
                    }
                    _ => {
                        return FilterResult::Error(
                            "Expression did not evaluate to a boolean".to_owned(),
                        )
                    }
                },
                Err(e) => return FilterResult::Error(format!("{}", e)),
            }
        }
        FilterResult::Retain
    }
}

///////////////////////////////////////////////////////////
// Evaluator context

trait FilterEvaluatorContext {
    fn get_value(&self, property: &Property) -> DuckResult<Constant>;
}

struct BuildFilterValidator {}

impl BuildFilterValidator {
    pub fn validate(expression: &Expression) -> DuckResult<()> {
        expression.accept(
            &BuildFilterValidator {},
            &FilterEvaluator::<BuildFilterValidator>::new(),
        )?;
        Ok(())
    }
}

impl FilterEvaluatorContext for BuildFilterValidator {
    fn get_value(&self, property: &Property) -> DuckResult<Constant> {
        Ok(match property {
            Property::Branch => Constant::String("".to_owned()),
            Property::Status => Constant::Status(BuildStatus::Unknown),
            Property::Definition => Constant::String("".to_owned()),
            Property::Project => Constant::String("".to_owned()),
            Property::Build => Constant::String("".to_owned()),
            Property::Collector => Constant::String("".to_owned()),
            Property::Provider => Constant::String("".to_owned()),
        })
    }
}

impl FilterEvaluatorContext for Build {
    fn get_value(&self, property: &Property) -> DuckResult<Constant> {
        Ok(match property {
            Property::Branch => Constant::String(self.branch.clone()),
            Property::Status => Constant::Status(self.status.clone()),
            Property::Definition => Constant::String(self.definition_id.clone()),
            Property::Project => Constant::String(self.project_id.clone()),
            Property::Build => Constant::String(self.build_id.clone()),
            Property::Collector => Constant::String(self.collector.clone()),
            Property::Provider => Constant::String(self.provider.clone()),
        })
    }
}

///////////////////////////////////////////////////////////
// Evaluator

struct FilterEvaluator<T: FilterEvaluatorContext> {
    _p: std::marker::PhantomData<T>,
}

impl<T: FilterEvaluatorContext> FilterEvaluator<T> {
    pub fn new() -> Self {
        Self {
            _p: std::marker::PhantomData,
        }
    }
}

impl<T: FilterEvaluatorContext> Visitor<T, Constant> for FilterEvaluator<T> {
    fn or(&self, ctx: &T, left: &Expression, right: &Expression) -> DuckResult<Constant> {
        let left = left.accept(ctx, self)?;
        let right = right.accept(ctx, self)?;
        match (&left, &right) {
            (Constant::Boolean(lhs), Constant::Boolean(rhs)) => Ok(Constant::Boolean(*lhs || *rhs)),
            _ => Err(format_err!("Mismatched types in OR expression.")),
        }
    }

    fn and(&self, ctx: &T, left: &Expression, right: &Expression) -> DuckResult<Constant> {
        let left = left.accept(ctx, self)?;
        let right = right.accept(ctx, self)?;
        match (left, right) {
            (Constant::Boolean(lhs), Constant::Boolean(rhs)) => Ok(Constant::Boolean(lhs && rhs)),
            _ => Err(format_err!("Mismatched types in AND expression.")),
        }
    }

    fn not(&self, ctx: &T, exp: &Expression) -> DuckResult<Constant> {
        match exp.accept(ctx, self)? {
            Constant::Boolean(e) => Ok(Constant::Boolean(!e)),
            _ => Err(format_err!("Can't negate expression")),
        }
    }

    fn constant(&self, _ctx: &T, constant: &Constant) -> DuckResult<Constant> {
        Ok(constant.clone())
    }

    fn property(&self, ctx: &T, property: &Property) -> DuckResult<Constant> {
        ctx.get_value(property)
    }

    fn scope(&self, ctx: &T, exp: &Expression) -> DuckResult<Constant> {
        Ok(exp.accept(ctx, self)?)
    }

    fn relational(
        &self,
        ctx: &T,
        left: &Expression,
        right: &Expression,
        operator: &Operator,
    ) -> DuckResult<Constant> {
        let left = left.accept(ctx, self)?;
        let right = right.accept(ctx, self)?;

        match operator {
            Operator::EqualTo => match (left, right) {
                (Constant::Integer(lhs), Constant::Integer(rhs)) => {
                    Ok(Constant::Boolean(lhs == rhs))
                }
                (Constant::Boolean(lhs), Constant::Boolean(rhs)) => {
                    Ok(Constant::Boolean(lhs == rhs))
                }
                (Constant::String(lhs), Constant::String(rhs)) => Ok(Constant::Boolean(lhs == rhs)),
                (Constant::Status(lhs), Constant::Status(rhs)) => Ok(Constant::Boolean(lhs == rhs)),
                _ => Err(format_err!("Mismatched types in '==' expression.")),
            },
            Operator::NotEqualTo => match (left, right) {
                (Constant::Integer(lhs), Constant::Integer(rhs)) => {
                    Ok(Constant::Boolean(lhs != rhs))
                }
                (Constant::Boolean(lhs), Constant::Boolean(rhs)) => {
                    Ok(Constant::Boolean(lhs != rhs))
                }
                (Constant::String(lhs), Constant::String(rhs)) => Ok(Constant::Boolean(lhs != rhs)),
                (Constant::Status(lhs), Constant::Status(rhs)) => Ok(Constant::Boolean(lhs != rhs)),
                _ => Err(format_err!("Mismatched types in '!=' expression.")),
            },
            Operator::GreaterThan => match (left, right) {
                (Constant::Integer(lhs), Constant::Integer(rhs)) => {
                    Ok(Constant::Boolean(lhs > rhs))
                }
                _ => Err(format_err!("Mismatched types in '>' expression.")),
            },
            Operator::GreaterThanOrEqualTo => match (left, right) {
                (Constant::Integer(lhs), Constant::Integer(rhs)) => {
                    Ok(Constant::Boolean(lhs >= rhs))
                }
                _ => Err(format_err!("Mismatched types in '>=' expression.")),
            },
            Operator::LessThan => match (left, right) {
                (Constant::Integer(lhs), Constant::Integer(rhs)) => {
                    Ok(Constant::Boolean(lhs < rhs))
                }
                _ => Err(format_err!("Mismatched types in '<' expression.")),
            },
            Operator::LessThanOrEqualTo => match (left, right) {
                (Constant::Integer(lhs), Constant::Integer(rhs)) => {
                    Ok(Constant::Boolean(lhs <= rhs))
                }
                _ => Err(format_err!("Mismatched types in '<=' expression.")),
            },
        }
    }
}

///////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builds::{BuildBuilder, BuildStatus};
    use crate::query;
    use test_case::test_case;

    #[test_case("3 == 3", Constant::Boolean(true) ; "integer_equal_to_1")]
    #[test_case("3 == 2", Constant::Boolean(false) ; "integer_equal_to_2")]
    #[test_case("3 != 2", Constant::Boolean(true) ; "integer_not_equal_to_1")]
    #[test_case("3 != 3", Constant::Boolean(false) ; "integer_not_equal_to_2")]
    #[test_case("3 > 2", Constant::Boolean(true) ; "integer_greater_than_1")]
    #[test_case("3 > 3", Constant::Boolean(false) ; "integer_greater_than_2")]
    #[test_case("3 >= 2", Constant::Boolean(true) ; "integer_greater_than_or_equal_to_1")]
    #[test_case("3 >= 3", Constant::Boolean(true) ; "integer_greater_than_or_equal_to_2")]
    #[test_case("3 >= 4", Constant::Boolean(false) ; "integer_greater_than_or_equal_to_3")]
    #[test_case("2 < 3", Constant::Boolean(true) ; "integer_less_than_1")]
    #[test_case("3 < 3", Constant::Boolean(false) ; "integer_less_than_2")]
    #[test_case("2 <= 3", Constant::Boolean(true) ; "integer_less_than_or_equal_to_1")]
    #[test_case("2 <= 2", Constant::Boolean(true) ; "integer_less_than_or_equal_to_2")]
    #[test_case("2 <= 1", Constant::Boolean(false) ; "integer_less_than_or_equal_to_3")]
    #[test_case("!true", Constant::Boolean(false) ; "negated_true_1")]
    #[test_case("NOT true", Constant::Boolean(false) ; "negated_true_2")]
    #[test_case("!false", Constant::Boolean(true) ; "negated_false_1")]
    #[test_case("NOT false", Constant::Boolean(true) ; "negated_false_2")]
    #[test_case("true and true", Constant::Boolean(true) ; "and_1")]
    #[test_case("true and false", Constant::Boolean(false) ; "and_2")]
    #[test_case("true or true", Constant::Boolean(true) ; "or_1")]
    #[test_case("true or false", Constant::Boolean(true) ; "or_2")]
    #[test_case("false or false", Constant::Boolean(false) ; "or_3")]
    #[test_case("false or true", Constant::Boolean(true) ; "or_4")]
    #[test_case("(1 > 2) or (2 > 1)", Constant::Boolean(true) ; "scoped_1")]
    #[test_case("(1 > 2) and (2 > 1)", Constant::Boolean(false) ; "scoped_2")]
    fn should_evaluate_expression(expression: &str, expected: Constant) {
        // Given
        let build = BuildBuilder::dummy().build().unwrap();
        let evaluator = FilterEvaluator::<Build>::new();
        let expression = query::parse(expression).unwrap();

        // When
        let result = expression.accept(&build, &evaluator).unwrap();

        // Then
        assert_eq!(expected, result);
    }

    #[test_case("branch == 'develop'", Constant::Boolean(true))]
    #[test_case("status == 'queued'", Constant::Boolean(true))]
    #[test_case("project == 'foo'", Constant::Boolean(true))]
    #[test_case("definition == 'bar'", Constant::Boolean(true))]
    #[test_case("build == '123'", Constant::Boolean(true))]
    #[test_case("collector == 'test'", Constant::Boolean(true))]
    #[test_case("provider == 'TeamCity'", Constant::Boolean(true))]
    fn should_evaluate_expression_with_property(expression: &str, expected: Constant) {
        // Given
        let evaluator = FilterEvaluator::<Build>::new();
        let expression = query::parse(expression).unwrap();
        let build = BuildBuilder::dummy()
            .branch("develop")
            .collector("test")
            .provider("TeamCity")
            .status(BuildStatus::Queued)
            .project_id("foo")
            .definition_id("bar")
            .build_id("123")
            .build()
            .unwrap();

        // When
        let result = expression.accept(&build, &evaluator).unwrap();

        // Then
        assert_eq!(expected, result);
    }
}
