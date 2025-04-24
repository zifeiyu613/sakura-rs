use sea_query::{Iden, Expr, Query, QueryBuilder};

/// 花花世界表模型
#[derive(Debug, Clone)]
pub struct ActivityHuahua {
    pub id: i64,
    pub huahua_name: String,
    pub flowers: f64,
    pub count_number: String,
    pub create_time: chrono::DateTime<chrono::Utc>,
    pub update_time: chrono::DateTime<chrono::Utc>,
    pub init_flower: f64,
    pub water_period: i32,
    pub compensate_flag: i8,
    pub sys_prop_id: i64,
    pub tax_num: i32,
    pub huahua_type: Option<i8>,
    pub lucky_add_num: i32,
    pub lucky_max_num: i32,
    pub lucky_ratio: f32,
    pub lucky_icon: String,
    pub ex: Option<String>,
}

/// 花花世界表结构
#[derive(Debug, Clone, Copy, Iden)]
pub enum TActivityHuahua {
    #[iden = "t_activity_huahua"]
    Table,
    Id,
    HuahuaName,
    Flowers,
    CountNumber,
    CreateTime,
    UpdateTime,
    InitFlower,
    WaterPeriod,
    CompensateFlag,
    SysPropId,
    TaxNum,
    HuahuaType,
    LuckyAddNum,
    LuckyMaxNum,
    LuckyRatio,
    LuckyIcon,
    Ex,
}

/// 花花类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HuahuaType {
    World = 1,    // 花花世界
    Debut = 2,    // 花花出道
    Planet = 3,   // 花花星球
}

/// 补偿开关枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompensateFlag {
    On = 1,     // 开启
    Off = 2,    // 关闭
}

impl ActivityHuahua {
    /// 创建花花世界配置
    pub fn new(
        huahua_name: impl Into<String>,
        flowers: f64,
        count_number: impl Into<String>,
        init_flower: f64,
        water_period: i32,
        compensate_flag: CompensateFlag,
        sys_prop_id: i64,
        tax_num: i32,
        huahua_type: HuahuaType,
        lucky_add_num: i32,
        lucky_max_num: i32,
        lucky_ratio: f32,
        lucky_icon: impl Into<String>,
        ex: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: 0, // 自增ID
            huahua_name: huahua_name.into(),
            flowers,
            count_number: count_number.into(),
            create_time: now,
            update_time: now,
            init_flower,
            water_period,
            compensate_flag: compensate_flag as i8,
            sys_prop_id,
            tax_num,
            huahua_type: Some(huahua_type as i8),
            lucky_add_num,
            lucky_max_num,
            lucky_ratio,
            lucky_icon: lucky_icon.into(),
            ex,
        }
    }

    /// 查询所有花花世界配置
    pub fn find_all<'a, DB: sea_query::DatabaseBackend>(
        query_builder: &'a impl QueryBuilder<DB>
    ) -> Result<Vec<Self>, sea_query::Error> {
        let query = Query::select()
            .from(TActivityHuahua::Table)
            .columns([
                TActivityHuahua::Id,
                TActivityHuahua::HuahuaName,
                TActivityHuahua::Flowers,
                TActivityHuahua::CountNumber,
                TActivityHuahua::CreateTime,
                TActivityHuahua::UpdateTime,
                TActivityHuahua::InitFlower,
                TActivityHuahua::WaterPeriod,
                TActivityHuahua::CompensateFlag,
                TActivityHuahua::SysPropId,
                TActivityHuahua::TaxNum,
                TActivityHuahua::HuahuaType,
                TActivityHuahua::LuckyAddNum,
                TActivityHuahua::LuckyMaxNum,
                TActivityHuahua::LuckyRatio,
                TActivityHuahua::LuckyIcon,
                TActivityHuahua::Ex,
            ])
            .to_owned();

        // 实际执行查询和结果解析需要根据具体的数据库驱动实现
        // 这里返回 Err 作为示例
        Err(sea_query::Error::Custom("需要实现查询执行逻辑".into()))
    }

    /// 按ID查询
    pub fn find_by_id<'a, DB: sea_query::DatabaseBackend>(
        query_builder: &'a impl QueryBuilder<DB>,
        id: i64
    ) -> Result<Option<Self>, sea_query::Error> {
        let query = Query::select()
            .from(TActivityHuahua::Table)
            .columns([
                TActivityHuahua::Id,
                TActivityHuahua::HuahuaName,
                TActivityHuahua::Flowers,
                TActivityHuahua::CountNumber,
                TActivityHuahua::CreateTime,
                TActivityHuahua::UpdateTime,
                TActivityHuahua::InitFlower,
                TActivityHuahua::WaterPeriod,
                TActivityHuahua::CompensateFlag,
                TActivityHuahua::SysPropId,
                TActivityHuahua::TaxNum,
                TActivityHuahua::HuahuaType,
                TActivityHuahua::LuckyAddNum,
                TActivityHuahua::LuckyMaxNum,
                TActivityHuahua::LuckyRatio,
                TActivityHuahua::LuckyIcon,
                TActivityHuahua::Ex,
            ])
            .and_where(Expr::col(TActivityHuahua::Id).eq(id))
            .to_owned();

        // 实际执行查询和结果解析需要根据具体的数据库驱动实现
        // 这里返回 Err 作为示例
        Err(sea_query::Error::Custom("需要实现查询执行逻辑".into()))
    }

    /// 按类型查询
    pub fn find_by_type<'a, DB: sea_query::DatabaseBackend>(
        query_builder: &'a impl QueryBuilder<DB>,
        huahua_type: HuahuaType
    ) -> Result<Vec<Self>, sea_query::Error> {
        let query = Query::select()
            .from(TActivityHuahua::Table)
            .columns([
                TActivityHuahua::Id,
                TActivityHuahua::HuahuaName,
                TActivityHuahua::Flowers,
                TActivityHuahua::CountNumber,
                TActivityHuahua::CreateTime,
                TActivityHuahua::UpdateTime,
                TActivityHuahua::InitFlower,
                TActivityHuahua::WaterPeriod,
                TActivityHuahua::CompensateFlag,
                TActivityHuahua::SysPropId,
                TActivityHuahua::TaxNum,
                TActivityHuahua::HuahuaType,
                TActivityHuahua::LuckyAddNum,
                TActivityHuahua::LuckyMaxNum,
                TActivityHuahua::LuckyRatio,
                TActivityHuahua::LuckyIcon,
                TActivityHuahua::Ex,
            ])
            .and_where(Expr::col(TActivityHuahua::HuahuaType).eq(huahua_type as i8))
            .to_owned();

        // 实际执行查询和结果解析需要根据具体的数据库驱动实现
        // 这里返回 Err 作为示例
        Err(sea_query::Error::Custom("需要实现查询执行逻辑".into()))
    }

    /// 插入新记录
    pub fn insert<'a, DB: sea_query::DatabaseBackend>(
        &self,
        query_builder: &'a impl QueryBuilder<DB>
    ) -> Result<i64, sea_query::Error> {
        let query = Query::insert()
            .into_table(TActivityHuahua::Table)
            .columns([
                TActivityHuahua::HuahuaName,
                TActivityHuahua::Flowers,
                TActivityHuahua::CountNumber,
                TActivityHuahua::CreateTime,
                TActivityHuahua::UpdateTime,
                TActivityHuahua::InitFlower,
                TActivityHuahua::WaterPeriod,
                TActivityHuahua::CompensateFlag,
                TActivityHuahua::SysPropId,
                TActivityHuahua::TaxNum,
                TActivityHuahua::HuahuaType,
                TActivityHuahua::LuckyAddNum,
                TActivityHuahua::LuckyMaxNum,
                TActivityHuahua::LuckyRatio,
                TActivityHuahua::LuckyIcon,
                TActivityHuahua::Ex,
            ])
            .values_panic([
                self.huahua_name.clone().into(),
                self.flowers.into(),
                self.count_number.clone().into(),
                self.create_time.into(),
                self.update_time.into(),
                self.init_flower.into(),
                self.water_period.into(),
                self.compensate_flag.into(),
                self.sys_prop_id.into(),
                self.tax_num.into(),
                self.huahua_type.unwrap_or(1).into(),
                self.lucky_add_num.into(),
                self.lucky_max_num.into(),
                self.lucky_ratio.into(),
                self.lucky_icon.clone().into(),
                self.ex.clone().unwrap_or_default().into(),
            ])
            .to_owned();

        // 实际执行插入和获取自增ID需要根据具体的数据库驱动实现
        // 这里返回 Err 作为示例
        Err(sea_query::Error::Custom("需要实现插入执行逻辑".into()))
    }

    /// 更新记录
    pub fn update<'a, DB: sea_query::DatabaseBackend>(
        &self,
        query_builder: &'a impl QueryBuilder<DB>
    ) -> Result<(), sea_query::Error> {
        let query = Query::update()
            .table(TActivityHuahua::Table)
            .values([
                (TActivityHuahua::HuahuaName, self.huahua_name.clone().into()),
                (TActivityHuahua::Flowers, self.flowers.into()),
                (TActivityHuahua::CountNumber, self.count_number.clone().into()),
                (TActivityHuahua::UpdateTime, chrono::Utc::now().into()),
                (TActivityHuahua::InitFlower, self.init_flower.into()),
                (TActivityHuahua::WaterPeriod, self.water_period.into()),
                (TActivityHuahua::CompensateFlag, self.compensate_flag.into()),
                (TActivityHuahua::SysPropId, self.sys_prop_id.into()),
                (TActivityHuahua::TaxNum, self.tax_num.into()),
                (TActivityHuahua::HuahuaType, self.huahua_type.unwrap_or(1).into()),
                (TActivityHuahua::LuckyAddNum, self.lucky_add_num.into()),
                (TActivityHuahua::LuckyMaxNum, self.lucky_max_num.into()),
                (TActivityHuahua::LuckyRatio, self.lucky_ratio.into()),
                (TActivityHuahua::LuckyIcon, self.lucky_icon.clone().into()),
                (TActivityHuahua::Ex, self.ex.clone().unwrap_or_default().into()),
            ])
            .and_where(Expr::col(TActivityHuahua::Id).eq(self.id))
            .to_owned();

        // 实际执行更新需要根据具体的数据库驱动实现
        // 这里返回 Err 作为示例
        Err(sea_query::Error::Custom("需要实现更新执行逻辑".into()))
    }

    /// 删除记录
    pub fn delete<'a, DB: sea_query::DatabaseBackend>(
        query_builder: &'a impl QueryBuilder<DB>,
        id: i64
    ) -> Result<(), sea_query::Error> {
        let query = Query::delete()
            .from_table(TActivityHuahua::Table)
            .and_where(Expr::col(TActivityHuahua::Id).eq(id))
            .to_owned();

        // 实际执行删除需要根据具体的数据库驱动实现
        // 这里返回 Err 作为示例
        Err(sea_query::Error::Custom("需要实现删除执行逻辑".into()))
    }
}
