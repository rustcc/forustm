use sapper::{ SapperModule, SapperRouter, Response, Request, Result as SapperResult };
use sapper_std::{ Context, render, SessionVal, PathParams };

pub struct SectionDetail;

impl SectionDetail {
    fn index(req: &mut Request) -> SapperResult<Response> {
        let mut web = Context::new();
        web.add("sectionName", &"综合讨论区");
        web.add("title", &"Rust在中国已经逐渐火起来啦！");
        res_html!("sectionDetail.html", web)
    }
}

impl SapperModule for SectionDetail {
    fn router(&self, router: &mut SapperRouter) -> SapperResult<()> {
        router.get("/section/", SectionDetail::index);

        Ok(())
    }
}
