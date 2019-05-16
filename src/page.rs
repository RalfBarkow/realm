use crate::mode::Mode;

pub trait Page: serde::ser::Serialize {
    fn realm_id(&self) -> &'static str;
    fn realm_config(&self) -> Result<serde_json::Value, failure::Error> {
        Ok(serde_json::to_value(self)?)
    }
    fn realm_json(&self) -> Result<serde_json::Value, failure::Error> {
        self.realm_config()
    }
    fn page_with_response(
        &self,
        req: &crate::Request,
        html: crate::HTML,
        mut resp: crate::Response,
    ) -> crate::Result {
        *resp.body_mut() = match Mode::detect(&req) {
            Mode::API => serde_json::to_string(&self.realm_json()?)?.into(),
            Mode::HTML => html.render(self.widget_spec()?)?,
            Mode::Layout => serde_json::to_string(&self.widget_spec()?)?.into(),
        };
        Ok(resp)
    }
    fn page(&self, req: &crate::Request, html: crate::HTML) -> crate::Result {
        self.page_with_response(req, html, crate::Response::new(vec![]))
    }
    fn widget_spec(&self) -> Result<crate::WidgetSpec, failure::Error> {
        Ok(crate::WidgetSpec {
            id: self.realm_id(),
            config: self.realm_config()?,
        })
    }
}